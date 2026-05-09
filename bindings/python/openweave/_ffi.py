import ctypes
import os
import sys

def _load_lib():
    # Fallback to local build path for examples
    paths = [
        os.path.join(os.path.dirname(__file__), "libopenweavec.so"),
        "../../../target/debug/libopenweavec.so",
        "../../../target/debug/libopenweavec.dylib",
        "../../../target/debug/openweave.dll"
    ]
    
    for p in paths:
        if os.path.exists(p):
            return ctypes.CDLL(os.path.abspath(p))
            
    # Try system paths
    lib_name = "libopenweavec.so" if sys.platform != "darwin" else "libopenweavec.dylib"
    if sys.platform == "win32":
        lib_name = "openweave.dll"
    return ctypes.CDLL(lib_name)

lib = _load_lib()

lib.ow_agent_create.argtypes = [ctypes.c_char_p]
lib.ow_agent_create.restype = ctypes.c_void_p

lib.ow_agent_run.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)]
lib.ow_agent_run.restype = ctypes.c_int

lib.ow_agent_destroy.argtypes = [ctypes.c_void_p]
lib.ow_agent_destroy.restype = None

lib.ow_free_string.argtypes = [ctypes.c_char_p]
lib.ow_free_string.restype = None