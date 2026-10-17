-- Original-only passive instrumentation. No state writes, button injection or
-- MMIO rereads. Program-fetch hooks record raw PC separately from hook address.
local root=[[__OUTPUT__]]
local out=assert(io.open(root..'/events.csv','w'))
out:write('frame,event,kind,address,value,call,game_state,room,x,y,held,pressed,projectile_count,slot,type,instruction,interrupts\n')
local sequence,calls,snapshots,completed_frames=0,0,0,0
local failed=false
local started,created,impact,finished=false,false,false,false
local slot=-1
local diagnostic_count,consumed_x_saved=0,false
local poll_active,poll_low,poll_high=false,nil,nil
local pending={}
local watched={}
local frame_limit,fire_frame=8850,8660
local hardware_read,hardware_write
local kinds={'fire','handle','draw-player','draw-bomb'}
local function word(a) return memory.readword('WRAM',a) end
local function problem(message)
 if not failed then
  failed=true
  local f=assert(io.open(root..'/failure.txt','w'))
  f:write(tostring(message),'\n');f:close()
 end
end
local function guard(fn)
 return function(...)
  if failed then return end
  local ok,message=pcall(fn,...)
  if not ok then problem(message) end
 end
end
local function reg(name)
 local value=memory.getregister(name)
 if type(value)=='boolean' then return value and 1 or 0 end
 assert(type(value)=='number','missing CPU register '..name)
 return value
end
local function emit(kind,address,value,call)
 sequence=sequence+1
 assert(sequence<=500000,'event limit exceeded')
 local active=pending[#pending]
 local current_type,current_instruction=0,0
 if slot>=0 then current_type=word(0xc18+slot*2);current_instruction=word(0xc40+slot*2) end
 local parts={movie.currentframe(),sequence,kind,address or 0,value or 0,call or 0,
  word(0x998),word(0x79b),word(0xaf6),word(0xafa),word(0x8b),word(0x8f),word(0xcce),
  slot,current_type,current_instruction,active and active.interrupts or 0}
 out:write(table.concat(parts,','),'\n')
 return sequence
end
local function snapshot(event,address,invocation,phase)
 snapshots=snapshots+1
 assert(snapshots<=512,'WRAM snapshot limit exceeded')
 local stem=root..'/states/state-'..event
 local f=assert(io.open(stem..'.wram','wb'))
 local bytes=memory.readregion('WRAM',0,131072)
 for i=0,131071,256 do f:write(string.char(table.unpack(bytes,i,i+255))) end
 f:close()
 local cpu=assert(io.open(stem..'.cpu.toml','w'))
 cpu:write('schema = "original-cpu-context-v1"\nphase = "',phase,'"\n')
 for _,name in ipairs({'a','x','y','s','d','db','pb','pc','p','e'}) do
  cpu:write(name,' = ',tostring(reg(name)),'\n')
 end
 cpu:write('boundary_pc = ',address,'\ncall = ',invocation.id,
  '\ncontinuation_pc = ',invocation.continuation,'\nentry_s = ',invocation.entry_s,
  '\nexpected_return_s = ',invocation.return_s,'\nexit_pc = ',invocation.exit_pc or 0,
  '\ninterrupts = ',invocation.interrupts,'\nsaved_status = ',invocation.saved_status or -1,'\n')
 cpu:close()
end
local function diagnostic(label,event,address,pad)
 diagnostic_count=diagnostic_count+1
 assert(diagnostic_count<=2,'diagnostic snapshot limit exceeded')
 local stem=root..'/diagnostics/'..label
 local file=assert(io.open(stem..'.wram','wb'))
 local bytes=memory.readregion('WRAM',0,131072)
 for i=0,131071,256 do file:write(string.char(table.unpack(bytes,i,i+255))) end
 file:close()
 local context=assert(io.open(stem..'.context.toml','w'))
 context:write('schema = "original-diagnostic-context-v1"\ncall_proof = false\n',
  'frame = ',movie.currentframe(),'\nevent = ',event,'\nhook_address = ',address,
  '\nactual_hardware_pad = ',pad or -1,'\n')
 for _,name in ipairs({'a','x','y','s','d','db','pb','pc','p','e'}) do
  context:write('cpu_',name,' = ',tostring(reg(name)),'\n')
 end
 for _,field in ipairs({
  {'game_state',0x998},{'room',0x79b},{'samus_x',0xaf6},{'samus_y',0xafa},
  {'held',0x8b},{'pressed',0x8f},{'previous',0x97},{'samus_filtered_new_input',0xe00},
  {'shoot_binding',0x9b2},{'equipped_beams',0x9a6},{'hud_selection',0x9d2},
  {'samus_pose',0xa1c},{'samus_input_handler',0xa60},{'samus_movement_handler',0xa58},
  {'hyper_beam',0xa76},{'ceres_flag',0x93f},{'cooldown',0xccc},{'projectile_count',0xcce},
  {'time_frozen',0xa78},{'health',0x9c2},{'flare_counter',0xcd0},
  {'samus_y_radius',0xb00},{'samus_y_speed',0xb2e},{'samus_y_subspeed',0xb2c},
  {'samus_alpha',0xa42},{'samus_beta',0xa44},{'elevator',0xe18},
  {'animation_speed_divisor',0xa66},{'grapple_function',0xd32},{'samus_filtered_held',0xdfe}
 }) do context:write(field[1],' = ',word(field[2]),'\n') end
 context:write('samus_movement_type = ',memory.readbyte('WRAM',0xa1f),'\n')
 context:write('samus_x_direction = ',memory.readbyte('WRAM',0xa1e),'\n')
 context:close()
end
local function all_empty()
 for i=0,9 do if word(0xc40+i*2)~=0 then return false end end
 return word(0xcce)==0
end
local function track()
 if not started or finished then return end
 if not created then
  for i=0,9 do
   if word(0xc40+i*2)~=0 and word(0xc18+i*2)==0x8000 then
    assert(slot==-1,'more than one ordinary projectile created')
    slot=i;created=true
   end
  end
 else
  local t=word(0xc18+slot*2)
  if t==0x8700 and word(0xc40+slot*2)~=0 then impact=true end
 end
end
local function hardware(kind,address,value)
 if not started or finished then return end
 -- Global callbacks also see mapped WRAM-domain events: retain BUS only.
 if address<0x1000000 or address>=0x2000000 then return end
 local bus=address-0x1000000
 local bank=math.floor(bus/65536)
 if bank%128>=64 then return end
 local low=bus%65536
 if low>=0x2100 and low<=0x21ff or low>=0x4200 and low<=0x421f or low>=0x4300 and low<=0x437f then
  -- Controller reads already have a complete reset-prefix ledger.
  if kind=='hardware-read' and (low==0x4218 or low==0x4219) then return end
  emit(kind,bus,value,pending[#pending] and pending[#pending].id or 0)
 end
end
hardware_read=guard(function(a,v) hardware('hardware-read',a,v) end)
hardware_write=guard(function(a,v) hardware('hardware-write',a,v) end)
local function on_return(address)
 local invocation=pending[#pending]
 if not invocation or invocation.continuation~=address or reg('s')~=invocation.return_s then return end
 assert(reg('pb')==math.floor(address/65536) and reg('pc')==(address+1)%65536,'return PC fetch convention differs')
 assert(invocation.exit_pc,'continuation reached without witnessed RTS/RTL')
 if invocation.kind=='fire' then assert(invocation.interrupts==0,'first Fire invocation interrupted; fixture context unsupported') end
 track()
 local event=emit(invocation.kind..'-return',address,0,invocation.id)
 snapshot(event,address,invocation,'return')
 pending[#pending]=nil
 if invocation.kind=='fire' then assert(created,'physical X did not create an ordinary beam') end
end
local function watch_return(address)
 if watched[address] then return end
 watched[address]=true
 memory.registerexec('BUS',address,guard(function() on_return(address) end))
end
local definitions={
 {kind='fire',address=0x90b887,offset=2,pop=3,exit_s=1,exits={0x90b89d,0x90b8a9,0x90b96d}},
 {kind='handle',address=0x90aece,offset=1,pop=2,exit_s=0,exits={0x90aef2}},
 {kind='draw-player',address=0x938254,offset=1,pop=3,exit_s=0,long=true,exits={0x9382fc}},
 {kind='draw-bomb',address=0x93834d,offset=1,pop=3,exit_s=0,long=true,exits={0x9383be}}
}
local function enter(definition)
 if definition.kind=='fire' and movie.currentframe()>=8640 then
  assert(not started,'more than one Fire invocation in the declared shot scenario')
  assert(movie.currentframe()>=fire_frame,'Fire occurred before the authored X press')
  assert(movie.currentframe()<=fire_frame+8,'Fire occurred outside the bounded input window')
  assert(word(0x998)==8 and word(0x79b)==0xdf45,'shot is not ordinary DF45 gameplay')
  assert(word(0xaf6)==130 and word(0xafa)==75
    and word(0xa1c)==1 and memory.readbyte('WRAM',0xa1e)==8
    and memory.readbyte('WRAM',0xa1f)==0 and word(0xb00)==21
    and word(0xb2c)==0 and word(0xb2e)==0,'post-turn right-standing Fire context differs')
  assert(word(0x9b2)==0x0040,'physical X no longer maps to shoot')
  assert(word(0xa76)==0 and word(0x9a6)==0 and word(0x93f)<0x8000,'unsupported beam or Ceres context')
  assert(all_empty(),'projectile pool was not empty before the one-shot scenario')
  started=true
  memory.registerread(nil,hardware_read)
  memory.registerwrite(nil,hardware_write)
 end
 if not started or finished then return end
 assert(reg('e')==0,'emulation-mode call stack unsupported')
 assert(reg('pb')==math.floor(definition.address/65536) and reg('pc')==(definition.address+1)%65536,'entry PC fetch convention differs')
 local s=reg('s')
 assert(s>=0x100 and s+definition.pop<0x2000,'entry stack outside mirrored WRAM')
 local lo=memory.readbyte('BUS',s+definition.offset)
 local hi=memory.readbyte('BUS',s+definition.offset+1)
 local bank=definition.long and memory.readbyte('BUS',s+3) or reg('pb')
 local continuation=bank*65536+(lo+hi*256+1)%65536
 calls=calls+1
 local invocation={kind=definition.kind,id=calls,entry_s=s,return_s=s+definition.pop,
  continuation=continuation,interrupts=0,definition=definition}
 if definition.kind=='fire' then invocation.saved_status=memory.readbyte('BUS',s+1) end
 pending[#pending+1]=invocation
 watch_return(continuation)
 track()
 local event=emit(definition.kind..'-entry',definition.address,0,invocation.id)
 snapshot(event,definition.address,invocation,'entry')
end
for _,definition in ipairs(definitions) do
 local definition=definition
 memory.registerexec('BUS',definition.address,guard(function() enter(definition) end))
 for _,address in ipairs(definition.exits) do
  local address=address
  memory.registerexec('BUS',address,guard(function()
   local invocation=pending[#pending]
   if invocation and invocation.kind==definition.kind then
    assert(reg('s')==invocation.entry_s+definition.exit_s,'RTS/RTL stack witness differs')
    invocation.exit_pc=address
    emit(definition.kind..'-exit',address,0,invocation.id)
   end
  end))
 end
end
local function boundary(kind,address)
 if kind=='controller-entry' then poll_active=true;poll_low=nil;poll_high=nil end
 if kind=='nmi-entry' or kind=='irq-entry' then
  for _,invocation in ipairs(pending) do invocation.interrupts=invocation.interrupts+1 end
 end
 track()
 local event=emit(kind,address,0,0)
 if kind=='controller-return' then
  if not consumed_x_saved and movie.currentframe()>=8640 and poll_active
    and poll_low~=nil and poll_high~=nil and poll_low+256*poll_high==0x0040
    and word(0x8b)==0x0040 and word(0x8f)==0x0040 then
   diagnostic('first-consumed-X',event,address,poll_low+256*poll_high)
   consumed_x_saved=true
  end
  poll_active=false
 end
 if kind=='dispatch-complete' and created and not finished and all_empty() then
  assert(#pending==0,'shot removed while a captured invocation remains active')
  assert(impact,'shot vanished without an observed impact/explosion stage')
  finished=true
  emit('shot-removed',address,0,0)
  memory.unregisterread(nil,hardware_read)
  memory.unregisterwrite(nil,hardware_write)
 end
end
for _,hook in ipairs({
 {'reset',0x00841c},{'reset',0x80841c},{'boot-clear',0x808482},
 {'controller-entry',0x809459},{'controller-return',0x809496},
 {'dispatch-entry',0x828948},{'dispatch-complete',0x82897a},
 {'explicit-wait',0x808338},{'nmi-entry',0x009583},{'nmi-entry',0x809583},
 {'irq-entry',0x00986a},{'irq-entry',0x80986a}
}) do
 local kind,address=hook[1],hook[2]
 memory.registerexec('BUS',address,guard(function() boundary(kind,address) end))
end
for _,hook in ipairs({
 {'plm-entry',0x8484e7},{'sfx1-max15-entry',0x809021},{'sfx2-max6-entry',0x8090cb}
}) do
 local kind,address=hook[1],hook[2]
 memory.registerexec('BUS',address,guard(function()
  if started and not finished then
   local call=pending[#pending] and pending[#pending].id or 0
   emit(kind,address,reg('a'),call)
   emit('semantic-status',address,reg('p'),call)
  end
 end))
end
for bank=0,191 do
 if bank%128<64 then
  for low=0x4218,0x4219 do
   local address=bank*65536+low
   memory.registerread('BUS',address,guard(function(a,v)
    if poll_active then
     if address%65536==0x4218 then poll_low=v else poll_high=v end
    end
    emit('controller-read',address,v,0)
   end))
  end
 end
end
function on_frame_emulated()
 completed_frames=completed_frames+1
 if not failed then
  local ok,message=pcall(function()
   local event=emit('frame-complete',0,0,0)
   if movie.currentframe()==8648 then
    diagnostic('before-turn-frame-8648',event,0,nil)
    assert(word(0xa1c)==0 and word(0xaf6)==128 and word(0xafa)==72
      and word(0xb00)==24 and word(0xa42)==0xe695 and word(0xa44)==0xe8aa
      and word(0x998)==8 and word(0x79b)==0xdf45,'pre-turn front-facing platform hypothesis differs')
   end
   assert(movie.currentframe()==completed_frames,'movie frame convention changed')
   assert(not started or finished or movie.currentframe()<=fire_frame+120,'shot lifecycle exceeded 120 original frames')
   out:flush()
  end)
  if not ok then problem(message) end
 end
 if movie.currentframe()==frame_limit then
  local file=assert(io.open(root..'/completion.txt','w'))
  file:write('completed_frames=',completed_frames,'\nlast_frame=',movie.currentframe(),
   '\nevents=',sequence,'\nsnapshots=',snapshots,'\nactive_calls=',#pending,
   '\nshot_created=',created and 1 or 0,'\nshot_impact=',impact and 1 or 0,
   '\nshot_removed=',finished and 1 or 0,'\nfailed=',failed and 1 or 0,
   '\ndiagnostic_snapshots=',diagnostic_count,'\n')
  file:close();out:flush()
 end
end
