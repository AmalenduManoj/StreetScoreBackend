# Tournament API Guide

This document covers only tournament-related APIs in this backend.

Base URL
- http://127.0.0.1:8080

Authentication
- Public endpoints: no token required
- Protected endpoints: require JWT token
- Header for protected endpoints:
  Authorization: Bearer <your_token>

Content Type
- Use Content-Type: application/json for request bodies

## What You Can Do With Tournament APIs

1. List all tournaments (public)
2. Get a tournament by id (public)
3. Create a tournament and register teams in one request (protected)
4. Get teams registered in a tournament (protected)
5. Add teams to an existing tournament (protected)
6. Remove teams from an existing tournament (protected)
7. Get tournaments for a team (protected)

## 1) Get all tournaments

Method
- GET

Path
- /tournaments

Auth
- Not required

Request body
- None

Response
- 200 OK
- JSON array of tournament objects

Example response
[
  {
    "id": 1,
    "name": "Summer Cup",
    "location": "Chennai",
    "start_date": "2026-05-20T10:00:00",
    "end_date": "2026-05-30"
  }
]

## 2) Get tournament by id

Method
- GET

Path
- /tournaments/{id}

Auth
- Not required

Path params
- id: integer tournament id

Request body
- None

Example
- /tournaments/1

Response
- 200 OK
- JSON tournament object

Example response
{
  "id": 1,
  "name": "Summer Cup",
  "location": "Chennai",
  "start_date": "2026-05-20T10:00:00",
  "end_date": "2026-05-30"
}

## 3) Create tournament with teams

Method
- POST

Path
- /tournaments

Auth
- Required

Request body payload
{
  "name": "Summer Cup",
  "location": "Chennai",
  "start_date": "2026-05-20T10:00:00",
  "end_date": "2026-05-30",
  "team_ids": [1, 2, 3]
}

Field details
- name: string
- location: string
- start_date: string datetime format
- end_date: string date format
- team_ids: array of team ids to register at creation time

Success response
- 200 OK
{
  "message": "Tournament created",
  "tournament_id": 12
}

Error responses
- 401 Unauthorized: Missing or invalid token
- 500 Internal Server Error: DB or insertion error

## 4) Get teams in a tournament

Method
- GET

Path
- /tournaments/{tournament_id}/teams

Auth
- Required

Path params
- tournament_id: integer

Request body
- None

Example
- /tournaments/12/teams

Response
- 200 OK
- JSON array (registered teams for the tournament)

## 5) Add teams to tournament

Method
- POST

Path
- /tournaments/{tournament_id}/teams

Auth
- Required

Path params
- tournament_id: integer

Request body payload
{
  "team_ids": [4, 5, 6]
}

Success response
- 200 OK
- Teams added to tournament successfully

Notes
- Duplicate team registration is ignored because of unique (team_id, tournament_id) handling

## 6) Remove teams from tournament

Method
- DELETE

Path
- /tournaments/{tournament_id}/teams

Auth
- Required

Path params
- tournament_id: integer

Request body payload
{
  "team_ids": [4, 5]
}

Success response
- 200 OK
- Teams removed from tournament successfully

Note
- Current delete logic removes rows created by the same authenticated user only

## 7) Get tournaments for a team

Method
- GET

Path
- /tournaments/team/{team_id}

Auth
- Required

Path params
- team_id: integer

Request body
- None

Example
- /tournaments/team/2

Response
- 200 OK
- JSON array of tournaments for that team

## Frontend Payload Quick Reference

Create tournament
{
  "name": "Summer Cup",
  "location": "Chennai",
  "start_date": "2026-05-20T10:00:00",
  "end_date": "2026-05-30",
  "team_ids": [1, 2, 3]
}

Add teams to tournament
{
  "team_ids": [4, 5, 6]
}

Remove teams from tournament
{
  "team_ids": [4, 5]
}

## Minimal UI Flow

1. Login and store JWT token
2. Call POST /tournaments with tournament details and initial team_ids
3. Later add teams with POST /tournaments/{tournament_id}/teams
4. Remove teams with DELETE /tournaments/{tournament_id}/teams
5. Read assignments with GET /tournaments/{tournament_id}/teams
