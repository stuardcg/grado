# 0003. Host on AWS Lambda behind a Function URL

- **Status:** Accepted
- **Date:** 2026-10-27
- **Ticket:** STU-36

Run the backend on AWS Lambda, exposed through a Function URL. The database is hosted outside AWS.

**Consequences:** the hosting platform is an adapter. One application router has two thin entry
points: a `lambda_http` binary for Lambda and a plain server binary for local runs or any
container host. API Gateway is not used, because its 30 s limit is too short for grading. The
Function URL is public (auth type `NONE`); the application's own authentication protects it. The
Lambda timeout is about 3 minutes, so a grading call fits in one request.
