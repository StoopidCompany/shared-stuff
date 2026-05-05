import { configureLogging, getLogger } from "stoopid-logging";

configureLogging({ level: "info" });

const log = getLogger("orders.api");
log.info("service started");
log.warn({ downstream: "billing", elapsedMs: 842 }, "downstream slow");

const bound = log.child({ requestId: "req_42" });
bound.info("processed request");
