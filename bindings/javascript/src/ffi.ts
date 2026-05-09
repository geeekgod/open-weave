import * as ffi from 'ffi-napi';
import * as ref from 'ref-napi';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

const stringPtr = ref.refType(ref.types.CString);

let libPath = '';
const devPaths = [
    path.join(__dirname, '../../../../target/debug/libopenweavec.dylib'),
    path.join(__dirname, '../../../../target/debug/libopenweavec.so'),
    path.join(__dirname, '../../../../target/debug/openweave.dll'),
];

for (const p of devPaths) {
    if (fs.existsSync(p)) {
        libPath = p;
        break;
    }
}

if (!libPath) {
    const ext = os.platform() === 'darwin' ? 'dylib' : os.platform() === 'win32' ? 'dll' : 'so';
    libPath = `libopenweavec.${ext}`;
}

export const lib = ffi.Library(libPath, {
    'ow_agent_create': ['pointer', ['string']],
    'ow_agent_run': ['int', ['pointer', 'string', ref.refType(stringPtr)]],
    'ow_agent_destroy': ['void', ['pointer']],
    'ow_free_string': ['void', ['string']]
});