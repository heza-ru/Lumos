console.log('[LUMOS]: Extended toolbar page preloading...');

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  invokeCloseApp:          () => ipcRenderer.invoke('close_app'),
  invokeDrawMode:          () => ipcRenderer.invoke('toggle_draw_or_pointer_window'),
  invokeDrawModeWithTool:  (tool)  => ipcRenderer.invoke('draw_mode_with_tool', tool),
  invokeSetPointerColor:   (index) => ipcRenderer.invoke('set_pointer_color', index),
});
