'use strict';

import { Context } from 'prototty';
const wasm = import('../wasm_out/app');

document.oncontextmenu = () => false;

wasm.then(async wasm => {
    let config = {
        WasmInputBufferType: wasm.InputBuffer,
        node: app_node,
        grid_width: 80,
        grid_height: 40,
        font_family: "PxPlus_IBM_CGA",
        font_size: "16px",
        cell_width_px: 17,
        cell_height_px: 17,
    };
    let storage_key = window.location.pathname + window.location.hash;
    console.log("Using storage key: ", storage_key);
    let context = await new Context(config).with_storage(storage_key);
    let app = new wasm.WebApp(context.grid(), context.storage());
    context.run_animation((input_buffer, period) => app.tick(input_buffer, period));
});
