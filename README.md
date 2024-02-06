# Flee AI Test

This is a test project I've put together to let others try interacting with and AI for a game I'm working on.

## TODO

- [x] Implement "Running" behavior (Make the agent head away from the player in the closest unobstructed direction)
  - [x] Implement basic [fleeing steering behavior](https://www.youtube.com/watch?v=Q4MU7pkDYmQ)
  - [x] Implement directional weighting from [this video](https://youtu.be/6BrZryMz-ac?t=115)
    - [x] Use raycasts to check when a direction is "obstructed"
- [ ] Implement "Hiding" behavior (Make the agent flee to a place where the player can't "see" it)
  - [x] Detect the players [complete field of view](https://ncase.me/sight-and-light/)
  - [ ] Check where the closest point is that is not in the players FOV
  - [ ] Pathfind there
- [ ] Blend between running and hiding depending on player proximity (with LOS)
