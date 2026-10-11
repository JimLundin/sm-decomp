void PortHostLoad(uint16 *v) {
 eproj_spawn_pt=(Point16U){v[0],v[1]};
 eproj_spawn_rect=(Rect16U){v[2],v[3],v[4],v[5]};
 eproj_spawn_r22=v[6]; eproj_spawn_varE24=v[7];
}
void PortHostSave(uint16 *v) {
 v[0]=eproj_spawn_pt.x; v[1]=eproj_spawn_pt.y;
 v[2]=eproj_spawn_rect.x; v[3]=eproj_spawn_rect.y;
 v[4]=eproj_spawn_rect.w; v[5]=eproj_spawn_rect.h;
 v[6]=eproj_spawn_r22; v[7]=eproj_spawn_varE24;
}
