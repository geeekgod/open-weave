import { Agent } from '../../bindings/javascript/dist';

async function main() {
    const agent = new Agent({
        system_prompt: "You are a research agent."
    });
    
    console.log("Running research agent...");
    try {
        const res = await agent.run("Research the history of Rust programming language");
        console.log(res);
    } catch (e) {
        console.error(e);
    } finally {
        agent.destroy();
    }
}

main();