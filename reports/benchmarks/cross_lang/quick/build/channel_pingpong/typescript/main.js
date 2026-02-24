const workerThreads = require("worker_threads");
const Worker = workerThreads.Worker;
const isMainThread = workerThreads.isMainThread;
const parentPort = workerThreads.parentPort;
const workerData = workerThreads.workerData;
const selfPath = process.argv[1] || "main.js";
if (!isMainThread && workerData && workerData.role === "pingpong") {
    parentPort.on("message", (token) => {
        parentPort.postMessage(token + 1);
    });
}
else {
    const rounds = 50000;
    const worker = new Worker(selfPath, {
        workerData: { role: "pingpong" },
    });
    let checksum = 0;
    let token = 1;
    (async function run() {
        for (let r = 0; r < rounds; r++) {
            worker.postMessage(token);
            token = await new Promise((resolve) => worker.once("message", resolve));
            checksum += token;
        }
        await worker.terminate();
        const ops = rounds;
        console.log("RESULT");
        console.log(checksum);
        console.log(ops);
    })().catch((error) => {
        console.error(error);
        process.exit(1);
    });
}
