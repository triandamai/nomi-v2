#!/usr/bin/env node
import { EXIT_PROBLEMS, run } from './run.js';

run(process.argv.slice(2), { cwd: process.cwd(), out: console.log, err: console.error }).then(
  (code) => process.exit(code),
  (error) => {
    console.error(error);
    process.exit(EXIT_PROBLEMS);
  }
);
