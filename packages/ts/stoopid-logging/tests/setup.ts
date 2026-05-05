import { readFileSync } from "node:fs";
import { resolve } from "node:path";

type ValidateFn = ((data: unknown) => boolean) & { errors?: unknown };
type AjvCtor = new (opts?: object) => { compile: (schema: object) => ValidateFn };

// eslint-disable-next-line @typescript-eslint/no-require-imports
const Ajv2020: AjvCtor = require("ajv/dist/2020.js");
// eslint-disable-next-line @typescript-eslint/no-require-imports
const addFormats: (a: object) => void = require("ajv-formats");

const SCHEMA_PATH = resolve(__dirname, "../../../../schemas/log-event.schema.json");

export const logEventSchema = JSON.parse(readFileSync(SCHEMA_PATH, "utf-8"));

const ajv = new Ajv2020({ strict: true });
addFormats(ajv);
export const validate = ajv.compile(logEventSchema);
