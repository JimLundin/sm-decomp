#include <stdint.h>
#include <stdlib.h>
#include <string.h>
extern int32_t sm_next_random_v1(uint8_t*,size_t);
int main(void) {
  uint8_t *p=calloc(1,131072); if(!p)return 2;
  if(sm_next_random_v1(0,131072)!=-1||sm_next_random_v1(p,1)!=-1||sm_next_random_v1(p,131073)!=-1)return 3;
  for(size_t i=0;i<131072;i++)if(p[i])return 4;
  if(sm_next_random_v1(p,131072)!=273||p[1509]!=17||p[1510]!=1)return 5;
  for(size_t i=0;i<131072;i++)if(i!=1509&&i!=1510&&p[i])return 6;
  free(p);return 0;
}
