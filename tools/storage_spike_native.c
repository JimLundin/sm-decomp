/* THROWAWAY storage spike: native writer costs and exact trace conversion.
 * Historical storage experiment. Uses installed zstd/zlib/OpenSSL only.
 * Supported grammar is the native port's decimal PPU/APU event profile.
 */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>
#include <errno.h>
#include <openssl/sha.h>
#include <zlib.h>

extern size_t ZSTD_compressBound(size_t);
extern size_t ZSTD_compress(void *, size_t, const void *, size_t, int);
extern size_t ZSTD_decompress(void *, size_t, const void *, size_t);
extern unsigned ZSTD_isError(size_t);
extern const char *ZSTD_versionString(void);

#define CAP (1024 * 1024)
#define MAX_FRAMES 1024
typedef struct { uint32_t frame, address; uint8_t kind, device, value; } Event;
typedef struct { uint32_t frame, offset; uint64_t ordinal; } Frame;
typedef struct {
  FILE *file;
  unsigned char buffer[CAP + 256];
  size_t used;
  uint64_t sequence, count, text_len, chunk_count, bytes;
  uint32_t previous, first, last;
  Frame frames[MAX_FRAMES]; size_t frame_count;
  SHA256_CTX text_hash;
  int level, pack;
} Writer;

static void fail(const char *s) { fprintf(stderr, "storage spike: %s\n", s); exit(2); }
static double clock_sec(clockid_t id) { struct timespec t; clock_gettime(id, &t); return t.tv_sec + t.tv_nsec / 1e9; }
static void write_bytes(FILE *f, const void *p, size_t n) { if (fwrite(p, 1, n, f) != n) fail("write"); }
static void u64(FILE *f, uint64_t x) { unsigned char b[8]; for (int i=0;i<8;i++) b[i]=(unsigned char)(x>>(8*i)); write_bytes(f,b,8); }
static void u32(FILE *f, uint32_t x) { unsigned char b[4]; for (int i=0;i<4;i++) b[i]=(unsigned char)(x>>(8*i)); write_bytes(f,b,4); }
static uint64_t get64(FILE *f) { unsigned char b[8]; if(fread(b,1,8,f)!=8) fail("truncated integer"); uint64_t x=0; for(int i=7;i>=0;i--)x=(x<<8)|b[i]; return x; }
static uint32_t get32(FILE *f) { unsigned char b[4]; if(fread(b,1,4,f)!=4) fail("truncated integer"); uint32_t x=0; for(int i=3;i>=0;i--)x=(x<<8)|b[i]; return x; }
static void hex(const unsigned char *d, char *s) { for (int i=0;i<32;i++)sprintf(s+2*i,"%02x",d[i]); }
static size_t format(Event e, char *b) {
  int n=e.kind ? snprintf(b,256,"apu %u %u %u\n",e.frame,e.address,e.value)
               : snprintf(b,256,"ppu %u %u %u %u\n",e.frame,e.device,e.address,e.value);
  if(n<0 || n>=256)fail("format"); return (size_t)n;
}
static uint32_t number(const char **p,char separator) {
  const char *start=*p;uint64_t value=0;if(**p<'0'||**p>'9')fail("numeric field");
  while(**p>='0'&&**p<='9'){value=value*10+(unsigned)(*(*p)++-'0');if(value>UINT32_MAX)fail("integer overflow");}
  if((*p-start>1&&*start=='0')||**p!=separator)fail("noncanonical numeric field");(*p)++;return (uint32_t)value;
}
static Event parse(const char *b) {
  Event e={0};const char *p=b+4;
  e.frame=number(&p,' ');
  if(!strncmp(b,"ppu ",4)){uint32_t device=number(&p,' '),address=number(&p,' '),value=number(&p,'\n');
    if(device>1||address>255||value>255||*p)fail("PPU field range");e.device=device;e.address=address;e.value=value;
  }else if(!strncmp(b,"apu ",4)){e.kind=1;e.address=number(&p,' ');uint32_t value=number(&p,'\n');if(value>255||*p)fail("APU field range");e.value=value;}
  else fail("unsupported event grammar");return e;
}
static size_t varint(unsigned char *,uint64_t);
static void flush_binary(Writer *);
static void literal_event(Writer *w,const char *text,size_t length) {
  unsigned frame;char tag[80];if(sscanf(text,"%79s %u",tag,&frame)!=2)fail("literal frame");
  if(w->count && (w->used+length+24>CAP || frame/64!=w->first/64 || frame<w->last))flush_binary(w);
  if(!w->count)w->first=frame;
  if(!w->frame_count || w->frames[w->frame_count-1].frame!=frame)w->frames[w->frame_count++]=(Frame){frame,(uint32_t)w->used,w->count};
  int64_t delta=(int64_t)frame-w->previous;
  w->used+=varint(w->buffer+w->used,delta>=0?(uint64_t)delta*2:(uint64_t)(-delta)*2-1);
  w->buffer[w->used++]=2;w->used+=varint(w->buffer+w->used,length);memcpy(w->buffer+w->used,text,length);w->used+=length;
  SHA256_Update(&w->text_hash,text,length);w->text_len+=length;w->previous=w->last=frame;w->count++;w->sequence++;
}
static size_t varint(unsigned char *b, uint64_t x) {
  size_t n=0;while(x>=128){b[n++]=(x&127)|128;x>>=7;}b[n++]=(unsigned char)x;return n;
}
static uint64_t unvarint(const unsigned char *b,size_t length,size_t *pos) {
  uint64_t x=0;for(unsigned shift=0;shift<70;shift+=7){if(*pos>=length)fail("truncated varint");
    unsigned v=b[(*pos)++];if(shift==63 && v>1)fail("varint overflow");x|=(uint64_t)(v&127)<<shift;
    if(!(v&128))return x;}fail("varint overflow");return 0;
}
static void flush_binary(Writer *w) {
  if(!w->count)return;
  size_t bound=ZSTD_compressBound(w->used);unsigned char *compressed=malloc(bound);if(!compressed)fail("alloc");
  size_t n=ZSTD_compress(compressed,bound,w->buffer,w->used,w->level);if(ZSTD_isError(n))fail("zstd compression");
  unsigned char logical[32],physical[32],text[32];SHA256_CTX hash;
  SHA256_Init(&hash);SHA256_Update(&hash,"E",1);SHA256_Update(&hash,w->buffer,w->used);SHA256_Final(logical,&hash);
  SHA256(compressed,n,physical);SHA256_Final(text,&w->text_hash);
  if(w->pack) {
    u64(w->file,w->first);u64(w->file,w->last);u64(w->file,w->sequence-w->count);u64(w->file,w->count);
    u64(w->file,w->text_len);u64(w->file,w->used);u64(w->file,n);u64(w->file,w->frame_count);
    write_bytes(w->file,logical,32);write_bytes(w->file,text,32);write_bytes(w->file,physical,32);
    for(size_t i=0;i<w->frame_count;i++){u32(w->file,w->frames[i].frame);u32(w->file,w->frames[i].offset);u64(w->file,w->frames[i].ordinal);}
  }
  write_bytes(w->file,compressed,n);w->bytes+=n;w->chunk_count++;
  w->used=w->count=w->text_len=w->frame_count=0;w->previous=0;SHA256_Init(&w->text_hash);free(compressed);
}
static void binary_event(Writer *w,Event e,const char *text,size_t text_len) {
  if(w->count && (w->used+16>CAP || e.frame/64!=w->first/64 || e.frame<w->last))flush_binary(w);
  if(!w->count)w->first=e.frame;
  if(!w->frame_count || w->frames[w->frame_count-1].frame!=e.frame) {
    if(w->frame_count>=MAX_FRAMES)fail("frame index capacity");
    w->frames[w->frame_count++]=(Frame){e.frame,(uint32_t)w->used,w->count};
  }
  int64_t delta=(int64_t)e.frame-w->previous;
  w->used+=varint(w->buffer+w->used,delta>=0 ? (uint64_t)delta*2 : (uint64_t)(-delta)*2-1);
  w->buffer[w->used++]=e.kind;
  if(!e.kind){w->buffer[w->used++]=e.device;w->buffer[w->used++]=(unsigned char)e.address;}
  else {for(int i=0;i<4;i++)w->buffer[w->used++]=(unsigned char)(e.address>>(8*i));}
  w->buffer[w->used++]=e.value;
  if(text){SHA256_Update(&w->text_hash,text,text_len);w->text_len+=text_len;}
  w->previous=w->last=e.frame;w->count++;w->sequence++;
}
static void flush_text(Writer *w) {
  if(!w->used)return;size_t bound=ZSTD_compressBound(w->used);void *compressed=malloc(bound);if(!compressed)fail("alloc");
  size_t n=ZSTD_compress(compressed,bound,w->buffer,w->used,w->level);if(ZSTD_isError(n))fail("zstd compression");
  unsigned char seal[32];SHA256(compressed,n,seal);write_bytes(w->file,compressed,n);w->bytes+=n;w->used=0;free(compressed);
}
static void finish(FILE *f) { if(fflush(f))fail("flush"); if(fsync(fileno(f)) && errno!=EINVAL)fail("fsync"); if(fclose(f))fail("close"); }

static int convert(const char *source,const char *dest,int level) {
  FILE *input=fopen(source,"rb"),*output=fopen(dest,"wb");if(!input||!output)fail("open");
  write_bytes(output,"SPTRACE1",8);Writer *w=calloc(1,sizeof(*w));if(!w)fail("alloc");
  w->file=output;w->level=level;w->pack=1;SHA256_Init(&w->text_hash);SHA256_CTX whole;SHA256_Init(&whole);
  double start=clock_sec(CLOCK_MONOTONIC),cpu=clock_sec(CLOCK_PROCESS_CPUTIME_ID);char line[256];uint64_t source_bytes=0;
  while(fgets(line,sizeof(line),input)){size_t n=strlen(line);
    if(!strncmp(line,"ppu ",4)||!strncmp(line,"apu ",4)){Event e=parse(line);binary_event(w,e,line,n);}
    else literal_event(w,line,n);
    SHA256_Update(&whole,line,n);source_bytes+=n;}
  if(ferror(input))fail("read");fclose(input);flush_binary(w);
  unsigned char seal[32];SHA256_Final(seal,&whole);u64(output,UINT64_MAX);u64(output,w->sequence);u64(output,source_bytes);u64(output,w->chunk_count);write_bytes(output,seal,32);
  finish(output);char identity[65]={0};hex(seal,identity);
  printf("mode = \"convert\"\nevents = %llu\nsource_bytes = %llu\nchunks = %llu\nwall_seconds = %.6f\ncpu_seconds = %.6f\nsource_sha256 = \"%s\"\nzstd_version = \"%s\"\n",
         (unsigned long long)w->sequence,(unsigned long long)source_bytes,(unsigned long long)w->chunk_count,
         clock_sec(CLOCK_MONOTONIC)-start,clock_sec(CLOCK_PROCESS_CPUTIME_ID)-cpu,identity,ZSTD_versionString());free(w);return 0;
}
static int verify(const char *source) {
  FILE *f=fopen(source,"rb");if(!f)fail("open");char magic[8];if(fread(magic,1,8,f)!=8||memcmp(magic,"SPTRACE1",8))fail("magic");
  SHA256_CTX whole;SHA256_Init(&whole);uint64_t total=0,text_total=0,chunks=0;
  double start=clock_sec(CLOCK_MONOTONIC),cpu=clock_sec(CLOCK_PROCESS_CPUTIME_ID);
  while(1){uint64_t first=get64(f);if(first==UINT64_MAX){uint64_t count=get64(f),length=get64(f),expected_chunks=get64(f);unsigned char expected[32],actual[32];
      if(fread(expected,1,32,f)!=32)fail("footer");SHA256_Final(actual,&whole);
      if(count!=total||length!=text_total||expected_chunks!=chunks||memcmp(expected,actual,32)||fgetc(f)!=EOF)fail("completion seal");break;}
    uint64_t last=get64(f),sequence=get64(f),count=get64(f),text_len=get64(f),raw_len=get64(f),packed_len=get64(f),frames=get64(f);
    if(raw_len>CAP+256||packed_len>ZSTD_compressBound(CAP+256)||frames>MAX_FRAMES||sequence!=total||first>UINT32_MAX||last>UINT32_MAX)fail("header bounds");
    unsigned char seals[96];if(fread(seals,1,96,f)!=96)fail("seals");
    Frame indices[MAX_FRAMES];for(uint64_t i=0;i<frames;i++){indices[i].frame=get32(f);indices[i].offset=get32(f);indices[i].ordinal=get64(f);}
    unsigned char *packed=malloc(packed_len),*raw=malloc(raw_len);if(!packed||!raw)fail("alloc");
    if(fread(packed,1,packed_len,f)!=packed_len)fail("truncated frame");unsigned char seal[32];SHA256(packed,packed_len,seal);if(memcmp(seal,seals+64,32))fail("physical seal");
    size_t n=ZSTD_decompress(raw,raw_len,packed,packed_len);if(ZSTD_isError(n)||n!=raw_len)fail("decode");
    SHA256_CTX logical;SHA256_Init(&logical);SHA256_Update(&logical,"E",1);SHA256_Update(&logical,raw,raw_len);SHA256_Final(seal,&logical);if(memcmp(seal,seals,32))fail("logical seal");
    SHA256_CTX text;SHA256_Init(&text);uint64_t seen=0,length=0;size_t pos=0,index_pos=0;int64_t frame=0;uint32_t prev_frame=0;char line[256];
    while(pos<raw_len){size_t record_offset=pos;uint64_t delta=unvarint(raw,raw_len,&pos);frame+=(delta&1)?-(int64_t)(delta/2)-1:(int64_t)(delta/2);
      if(frame<0||frame>UINT32_MAX||pos>=raw_len)fail("frame");Event e={0};e.frame=(uint32_t)frame;e.kind=raw[pos++];size_t need=e.kind==0?3:5;
      if(e.kind>2||(e.kind<2 && pos+need>raw_len))fail("record length");
      if(!seen || e.frame!=prev_frame){if(index_pos>=frames||indices[index_pos].frame!=e.frame||indices[index_pos].offset!=record_offset||indices[index_pos].ordinal!=seen)fail("frame index");index_pos++;}
      size_t size;
      if(e.kind==2){size=unvarint(raw,raw_len,&pos);if(size>=256||pos+size>raw_len)fail("literal length");memcpy(line,raw+pos,size);line[size]=0;pos+=size;}
      else{if(!e.kind){e.device=raw[pos++];e.address=raw[pos++];}else{for(int i=0;i<4;i++)e.address|=(uint32_t)raw[pos++]<<(8*i);}e.value=raw[pos++];size=format(e,line);}
      SHA256_Update(&text,line,size);SHA256_Update(&whole,line,size);length+=size;seen++;prev_frame=e.frame;
      if(seen==1 && e.frame!=first)fail("first frame");}
    SHA256_Final(seal,&text);if(memcmp(seal,seals+32,32)||seen!=count||length!=text_len||prev_frame!=last||index_pos!=frames)fail("text or counts");
    total+=seen;text_total+=length;chunks++;free(raw);free(packed);
  }fclose(f);printf("mode = \"verify\"\nevents = %llu\nsource_bytes = %llu\nchunks = %llu\nwall_seconds = %.6f\ncpu_seconds = %.6f\n",
       (unsigned long long)total,(unsigned long long)text_total,(unsigned long long)chunks,clock_sec(CLOCK_MONOTONIC)-start,clock_sec(CLOCK_PROCESS_CPUTIME_ID)-cpu);return 0;
}
static int bench(const char *sample,const char *mode,const char *dest,uint64_t events,int level) {
  FILE *f=fopen(sample,"rb");if(!f)fail("sample open");size_t capacity=1000000,count=0;Event *input=malloc(capacity*sizeof(Event));if(!input)fail("alloc");char line[256];
  while(count<capacity && fgets(line,sizeof(line),f))input[count++]=parse(line);fclose(f);if(!count)fail("empty sample");
  FILE *output=NULL;gzFile gz=NULL;Writer *w=calloc(1,sizeof(*w));if(!w)fail("alloc");w->level=level;SHA256_Init(&w->text_hash);
  if(!strcmp(mode,"gzip")){gz=gzopen(dest,"wb1");if(!gz)fail("gzip open");}
  else {output=fopen(dest,"wb");if(!output)fail("output open");w->file=output;}
  int code=!strcmp(mode,"binary")?1:!strcmp(mode,"plain")?2:!strcmp(mode,"gzip")?3:!strcmp(mode,"zstd-text")?4:!strcmp(mode,"null")?0:-1;
  if(code<0)fail("mode");
  if(code==1)w->pack=1; /* Include per-chunk hashes and frame index write cost. */
  double start=clock_sec(CLOCK_MONOTONIC),cpu=clock_sec(CLOCK_PROCESS_CPUTIME_ID);volatile uint64_t sink=0;
  for(uint64_t i=0;i<events;i++){Event e=input[i%count];
    if(code==1)binary_event(w,e,NULL,0);
    else if(code==2){if(e.kind)fprintf(output,"apu %u %u %u\n",e.frame,e.address,e.value);else fprintf(output,"ppu %u %u %u %u\n",e.frame,e.device,e.address,e.value);}
    else if(code==3){if(e.kind)gzprintf(gz,"apu %u %u %u\n",e.frame,e.address,e.value);else gzprintf(gz,"ppu %u %u %u %u\n",e.frame,e.device,e.address,e.value);}
    else if(code==4){size_t n=format(e,line);if(w->used+n>CAP)flush_text(w);memcpy(w->buffer+w->used,line,n);w->used+=n;}
    else sink+=e.value;
  }
  if(!strcmp(mode,"binary"))flush_binary(w);if(!strcmp(mode,"zstd-text"))flush_text(w);
  if(gz){if(gzclose(gz)!=Z_OK)fail("gzip close");output=fopen(dest,"rb");if(!output)fail("gzip fsync open");}
  finish(output);struct stat st;if(stat(dest,&st))fail("stat");
  printf("mode = \"%s\"\nevents = %llu\nbytes = %llu\nwall_seconds = %.6f\ncpu_seconds = %.6f\nlevel = %d\nloaded_events = %zu\nsink = %llu\n",mode,(unsigned long long)events,(unsigned long long)st.st_size,clock_sec(CLOCK_MONOTONIC)-start,clock_sec(CLOCK_PROCESS_CPUTIME_ID)-cpu,level,count,(unsigned long long)sink);
  free(w);free(input);return 0;
}
int main(int argc,char **argv) {
  if(argc==5 && !strcmp(argv[1],"convert"))return convert(argv[2],argv[3],atoi(argv[4]));
  if(argc==3 && !strcmp(argv[1],"verify"))return verify(argv[2]);
  if(argc==7 && !strcmp(argv[1],"bench"))return bench(argv[2],argv[3],argv[4],strtoull(argv[5],NULL,10),atoi(argv[6]));
  fail("usage: convert input output level | verify pack | bench sample mode output events level");return 2;
}
