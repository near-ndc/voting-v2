import os
import subprocess
import psycopg2
import argparse

# Set up argument parsing
parser = argparse.ArgumentParser(description='Load data into PostgreSQL from compressed files.')
parser.add_argument('--dbname', help='Database name', default=os.getenv('DB_NAME'))
parser.add_argument('--user', help='Database user', default=os.getenv('DB_USER'))
parser.add_argument('--password', help='Database password', default=os.getenv('DB_PASSWORD'))
parser.add_argument('--host', help='Database host', default=os.getenv('DB_HOST'))
parser.add_argument('--table', help='Target table name', default=os.getenv('TABLE_NAME'))
parser.add_argument('--path', help='Path to the directory containing the files', default=os.getenv('FILES_PATH'))

args = parser.parse_args()

# Function to process each file
def process_file(file_path, cursor, table_name):
    with subprocess.Popen(['gunzip', '-c', file_path], stdout=subprocess.PIPE) as proc:
        cursor.copy_expert(f"COPY {table_name} FROM STDIN WITH (FORMAT csv, HEADER true, DELIMITER ',')", proc.stdout)

# Main function to connect to the database and process files
def main(db_params, directory, table_name):
    conn = psycopg2.connect(**db_params)
    conn.autocommit = False
    cursor = conn.cursor()

    # Get a sorted list of files without the .gz extension and fitting the pattern
    files = sorted([f for f in os.listdir(directory)])
    total_files = len(files)
    processed_files = 0
    latest_file = None

    try:
        for file_name in files:
            latest_file = file_name
            file_path = os.path.join(directory, file_name)
            process_file(file_path, cursor, table_name)
            conn.commit()
            processed_files += 1
            print(f"Finished file: {processed_files}/{total_files} ({file_name})")

        print(f"Total processed files: {processed_files}/{total_files}")

    except Exception as e:
        conn.rollback()
        print(f"An error occurred: {e} at file {latest_file}")
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
    main(db_params, args.path, args.table)
