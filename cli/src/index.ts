#!/usr/bin/env node
import { Command } from 'commander';
import * as dotenv from 'dotenv';
import { resolve } from 'path';
import { z } from 'zod';

dotenv.config({ path: resolve(__dirname, '../.env') });

const program = new Command();

program
  .name('orbitstream')
  .description('CLI to deploy and manage OrbitStream continuous funding contracts on Soroban')
  .version('0.1.0');

const initSchema = z.object({
  receiver: z.string().min(1, "Receiver is required"),
  token: z.string().min(1, "Token is required"),
  flowRate: z.coerce.number().positive("Flow rate must be a positive number")
});

program
  .command('init')
  .description('Initialize a new funding stream configuration')
  .requiredOption('-r, --receiver <address>', 'The Stellar address of the open-source maintainer')
  .requiredOption('-t, --token <address>', 'The contract address of the token (e.g., USDC)')
  .requiredOption('-f, --flow-rate <amount>', 'The amount of tokens to stream per second')
  .action((options) => {
    const deployerSecret = process.env.SOROBAN_SECRET_KEY;
    if (!deployerSecret) {
      console.error("FATAL: SOROBAN_SECRET_KEY is missing from the .env file.");
      console.error("For your security, OrbitStream refuses to accept secret keys via command-line arguments.");
      process.exit(1);
    }

    const streamContractId = process.env.STREAM_CONTRACT_ID;
    if (!streamContractId) {
      console.error("FATAL: STREAM_CONTRACT_ID is missing from the .env file.");
      console.error("Please run the deployment script first.");
      process.exit(1);
    }
    console.log(`Targeting Stream Contract: ${streamContractId}`);

    const parsed = initSchema.safeParse(options);
    if (!parsed.success) {
      console.error("FATAL: Invalid CLI arguments.");
      console.error(parsed.error.format());
      process.exit(1);
    }

    console.log(`Starting OrbitStream initialization...`);
    console.log(`Receiver: ${parsed.data.receiver}`);
    console.log(`Token: ${parsed.data.token}`);
    console.log(`Flow Rate: ${parsed.data.flowRate}`);
    console.log(`SUCCESS: Dry-run configuration validated securely.`);
  });

program.parse(process.argv);
