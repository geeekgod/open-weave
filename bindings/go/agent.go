package openweave

/*
#cgo LDFLAGS: -L../../target/debug -lopenweavec
#include <stdlib.h>

extern void* ow_agent_create(const char* config_json);
extern int ow_agent_run(void* handle, const char* input, char** out_result);
extern void ow_agent_destroy(void* handle);
extern void ow_free_string(char* s);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"unsafe"
)

type Agent struct {
	handle unsafe.Pointer
}

type AgentConfig struct {
	LLM          string `json:"llm,omitempty"`
	SystemPrompt string `json:"system_prompt,omitempty"`
}

func New(config AgentConfig) (*Agent, error) {
	if config.LLM == "" {
		config.LLM = "openai/gpt-4o"
	}
	if config.SystemPrompt == "" {
		config.SystemPrompt = "You are a helpful AI assistant."
	}

	configBytes, err := json.Marshal(config)
	if err != nil {
		return nil, err
	}

	cConfig := C.CString(string(configBytes))
	defer C.free(unsafe.Pointer(cConfig))

	handle := C.ow_agent_create(cConfig)
	if handle == nil {
		return nil, fmt.Errorf("failed to create agent")
	}

	return &Agent{handle: handle}, nil
}

func (a *Agent) Run(prompt string) (string, error) {
	cPrompt := C.CString(prompt)
	defer C.free(unsafe.Pointer(cPrompt))

	var cOut *C.char
	res := C.ow_agent_run(a.handle, cPrompt, &cOut)
	if res != 0 {
		return "", fmt.Errorf("agent run failed with code %d", int(res))
	}

	outStr := C.GoString(cOut)
	C.ow_free_string(cOut)

	return outStr, nil
}

func (a *Agent) Destroy() {
	if a.handle != nil {
		C.ow_agent_destroy(a.handle)
		a.handle = nil
	}
}