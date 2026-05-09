import json
import ctypes
from ._ffi import lib

class Agent:
    def __init__(self, llm="openai/gpt-4o", system_prompt="You are a helpful AI assistant.", **kwargs):
        config = {
            "llm": llm,
            "system_prompt": system_prompt,
            **kwargs
        }
        config_str = json.dumps(config).encode('utf-8')
        self._handle = lib.ow_agent_create(config_str)
        if not self._handle:
            raise RuntimeError("Failed to create agent handle")

    def run(self, prompt: str) -> str:
        out_ptr = ctypes.c_char_p()
        prompt_bytes = prompt.encode('utf-8')
        res = lib.ow_agent_run(self._handle, prompt_bytes, ctypes.byref(out_ptr))
        
        if res != 0:
            raise RuntimeError(f"Agent execution failed with code {res}")
            
        result_str = out_ptr.value.decode('utf-8')
        lib.ow_free_string(out_ptr)
        return result_str

    def __del__(self):
        if hasattr(self, '_handle') and self._handle:
            lib.ow_agent_destroy(self._handle)