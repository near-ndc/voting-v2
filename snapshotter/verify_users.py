import os
import psycopg2
import argparse
import json

# Set up argument parsing
parser = argparse.ArgumentParser(description='Check records in PostgreSQL from a JSON file.')
parser.add_argument('--dbname', help='Database name', default=os.getenv('DB_NAME'))
parser.add_argument('--user', help='Database user', default=os.getenv('DB_USER'))
parser.add_argument('--password', help='Database password', default=os.getenv('DB_PASSWORD'))
parser.add_argument('--host', help='Database host', default=os.getenv('DB_HOST'))
parser.add_argument('--table', help='Target table name', default=os.getenv('TABLE_NAME'))
parser.add_argument('--json', help='Path to the JSON file', default=os.getenv('JSON_PATH'))
parser.add_argument('--column', help='Column name in the table', default=os.getenv('COLUMN_NAME'))

args = parser.parse_args()

# Function to check if records exist for each name in the JSON file
def check_records(cursor, table_name, column, json_path):
    # Read the JSON file and extract the names
    with open(json_path, 'r') as file:
        data = json.load(file)
        names = list(data.keys())

    # Check for each name in the database
    for name in names:
        cursor.execute(f"SELECT EXISTS(SELECT 1 FROM {table_name} WHERE {column} = %s)", (name,))
        exists = cursor.fetchone()[0]

        if exists:
            print(f"Record for '{name}' exists in the database.")
        else:
            print(f"Record for '{name}' does NOT exist in the database.")

# Main function to connect to the database and check the records
def main(db_params, table_name, column, json_path):
    conn = psycopg2.connect(**db_params)
    cursor = conn.cursor()

    try:
        check_records(cursor, table_name, column, json_path)
    except Exception as e:
        print(f"An error occurred: {e}")
    finally:
        cursor.close()
        conn.close()

if __name__ == "__main__":
    db_params = {
        'dbname': args.dbname,
        'user': args.user,
        'password': args.password,
        'host': args.host
    }
    main(db_params, args.table, args.column, args.json)
