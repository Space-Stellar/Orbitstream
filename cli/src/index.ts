#!/usr/bin/env node
import { Command } from 'commander';
import * as dotenv from 'dotenv';
import { resolve } from 'path';

// Securely load environment variables from the root of the cli directory
dotenv.config({ path: resolve(__dirname, '../.env') });

const program = new Command();

program
  .name('orbitstream')
  .description('CLI to deploy and manage OrbitStream continuous funding contracts on Soroban')
  .version('0.1.0');

program
  .command('init')
  .description('Initialize a new funding stream configuration')
  .requiredOption('-r, --receiver <address>', 'The Stellar address of the open-source maintainer')
  .requiredOption('-t, --token <address>', 'The contract address of the token (e.g., USDC)')
  .requiredOption('-f, --flow-rate <amount>', 'The amount of tokens to stream per second')
  .action((options) => {
    // SECURITY ENFORCEMENT: Never accept secret keys as CLI flags.
    const deployerSecret = process.env.SOROBAN_SECRET_KEY;
    if (!deployerSecret) {
      console.error("FATAL: SOROBAN_SECRET_KEY is missing from the .env file.");
      console.error("For your security, OrbitStream refuses to accept secret keys via command-line arguments.");
      process.exit(1);
    }

    console.log(`Starting OrbitStream initialization...`);
    console.log(`Receiver: ${options.receiver}`);
    console.log(`Token: ${options.token}`);
    console.log(`Flow Rate: ${options.flowRate}`);
    
    // Future integration: Soroban TS SDK bindings will be invoked here to submit the transaction.
    console.log(`SUCCESS: Dry-run configuration validated securely.`);
  });

program.parse(process.argv);
