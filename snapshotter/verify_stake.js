import pkg from 'pg';
const { Client } = pkg;
import * as fs from 'fs';
import { program } from 'commander';
import { PromisePool } from '@supercharge/promise-pool'

program
  .description('Check records in PostgreSQL from a JSON file.')
  .option('--dbname <type>', 'Database name', process.env.DB_NAME)
  .option('--user <type>', 'Database user', process.env.DB_USER)
  .option('--password <type>', 'Database password', process.env.DB_PASSWORD)
  .option('--host <type>', 'Database host', process.env.DB_HOST)
  .option('--table <type>', 'Target table name', process.env.TABLE_NAME)
  .option('--json <type>', 'Path to the JSON file', process.env.JSON_PATH)
  .option('--column <type>', 'Column name in the table', process.env.COLUMN_NAME);

program.parse(process.argv);
const options = program.opts();


// Function to check if records exist for each name in the JSON file
async function checkRecords(client, tableName, column, jsonPath) {
  // Read the JSON file and extract the names
  const data = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));
  const names = Object.keys(data);

  // Check for each name in the database using PromisePool
  const { results, errors } = await PromisePool
    .withConcurrency(8)
    .for(names)
    .process(async (name) => {
      const query = `SELECT EXISTS(SELECT 1 FROM ${tableName} WHERE ${column} = $1)`;
      const res = await client.query(query, [name]);
      const exists = res.rows[0].exists;

      if (exists) {
        console.log(`Record for '${name}' exists in the database.`);
      } else {
        console.log(`Record for '${name}' does NOT exist in the database.`);
      }
      return exists;
    });

  if (errors.length > 0) {
    errors.forEach(error => {
      console.error(`Error occurred: ${error.message}`);
    });
  }

  return results;
}

// Main function to connect to the database and check the records
async function main(dbParams, tableName, column, jsonPath) {
  const client = new Client(dbParams);

  try {
    await client.connect();
    await checkRecords(client, tableName, column, jsonPath);
  } catch (e) {
    console.error(`An error occurred: ${e.message}`);
  } finally {
    await client.end();
  }
}

// Execute the main function
const dbParams = {
  database: options.dbname,
  user: options.user,
  password: options.password,
  host: options.host
};

main(dbParams, options.table, options.column, options.json);
