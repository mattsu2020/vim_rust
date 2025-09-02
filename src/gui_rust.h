#ifndef GUI_RUST_H
#define GUI_RUST_H

#ifdef __cplusplus
extern "C" {
#endif

void rs_gui_run(void);
void rs_gui_gtk_event_loop(void);
void rs_gui_motif_event_loop(void);

#ifdef __cplusplus
}
#endif

#endif // GUI_RUST_H
