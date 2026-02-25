// @ts-nocheck
declare const require: any;
declare const process: any;

const workerThreads = require("worker_threads");
const Worker = workerThreads.Worker;
const isMainThread = workerThreads.isMainThread;
const parentPort = workerThreads.parentPort;
const workerData = workerThreads.workerData;

const selfPath = process.argv[1] || "main.js";

if (!isMainThread && workerData && workerData.role === "reduce") {
  const start = Number(workerData.start);
  const step = Number(workerData.step);
  const limit = Number(workerData.limit);
  let localSum = 0;
  for (let i = start; i < limit; i += step) {
    localSum += i + 1;
  }
  parentPort.postMessage(localSum);
} else {
  const workers = 4;
  const limit = 60000;
  const workerPromises: Promise<number>[] = [];
  for (let w = 0; w < workers; w++) {
    const worker = new Worker(selfPath, {
      workerData: { role: "reduce", start: w, step: workers, limit },
    });
    workerPromises.push(new Promise<number>((resolve) => worker.once("message", resolve)));
  }
  Promise.all(workerPromises)
    .then((partials) => {
      const checksum = partials.reduce((a, b) => a + b, 0);
      const ops = limit;
      console.log("RESULT");
      console.log(checksum);
      console.log(ops);
    })
    .catch((error: unknown) => {
      console.error(error);
      process.exit(1);
    });
}
