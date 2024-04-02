# Snapshot API

This is a Node.js application that provides an API endpoint for retrieving paginated, sorted, and filtered snapshot data.

## Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   ```

2. Install the dependencies:
   ```
   npm i
   ```

3. Start the application:
   ```
   npm start:local
   ```

## API Endpoint

The API endpoint is available at `/api/snapshot`.

### Query Parameters

The following query parameters can be used to customize the API response:

- `page` (optional, default: 0): Page number of the paginated results.
- `limit` (optional, default: 100): Maximum number of items per page.
- `sortBy` (optional, default: "name"): Field to sort the results by.
  - Possible values: "name", "stake", "active_months", "vote_power", "stake_power", "activity_power"
- `sortOrder` (optional, default: "asc"): Order in which the results should be sorted.
  - Possible values: "asc" (ascending), "desc" (descending)
- `prefix` (optional, default: ""): Prefix to filter the `account_id` field by.

### Example Requests

- `/api/snapshot`: First page of data with default sorting and no filtering.
- `/api/snapshot?page=1&limit=50`: Second page of data with 50 items per page.
- `/api/snapshot?sortBy=stake&sortOrder=desc`: First page of data sorted by stake in descending order.
- `/api/snapshot?page=2&limit=20&sortBy=vote_power&sortOrder=asc`: Third page of data sorted by vote power in ascending order with 20 items per page.
- `/api/snapshot?prefix=00&sortBy=activity_power&sortOrder=desc`: First page of data filtered by `account_id` prefix "00" and sorted by activity power in descending order.

### Error Handling

If an invalid `sortBy` parameter is provided, the API will respond with a 400 status code and an error message indicating the invalid parameter.

## Configuration

The application can be configured using the following environment variables:

- `SERVER_PORT` (default: 3000): The port on which the server will run.
- `SNAPSHOT_FILE` (default: 'snapshot.json'): The path to the snapshot JSON file.
