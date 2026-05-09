import { lib } from './ffi';
import * as ref from 'ref-napi';

export interface AgentConfig {
    llm?: string;
    system_prompt?: string;
    [key: string]: any;
}

export class Agent {
    private handle: Buffer;

    constructor(config: AgentConfig = {}) {
        const configStr = JSON.stringify({
            llm: config.llm || "openai/gpt-4o",
            system_prompt: config.system_prompt || "You are a helpful AI assistant.",
            ...config
        });
        
        this.handle = lib.ow_agent_create(configStr);
        if (this.handle.isNull()) {
            throw new Error("Failed to create agent");
        }
    }

    async run(prompt: string): Promise<string> {
        return new Promise((resolve, reject) => {
            const outPtr = ref.alloc(ref.refType(ref.types.CString));
            
            // In a real implementation this should use lib.ow_agent_run.async
            const res = lib.ow_agent_run(this.handle, prompt, outPtr);
            
            if (res !== 0) {
                reject(new Error(`Agent execution failed with code ${res}`));
                return;
            }

            const strPtr = outPtr.deref();
            const resultStr = strPtr.readCString();
            lib.ow_free_string(strPtr);
            
            resolve(resultStr);
        });
    }

    destroy() {
        if (this.handle && !this.handle.isNull()) {
            lib.ow_agent_destroy(this.handle);
        }
    }
}