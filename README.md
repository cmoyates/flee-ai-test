# Flee AI Test

This is a test project I've put together to let others try interacting with and AI for a game I'm working on.

## TODO

- [x] Implement "Fleeing" behavior (Make the agent head away from the player in the closest unobstructed direction)
  - [x] Implement basic [fleeing steering behavior](https://www.youtube.com/watch?v=Q4MU7pkDYmQ)
  - [x] Implement [directional weighting](https://youtu.be/6BrZryMz-ac?t=115)
    - [x] Use raycasts to check when a direction is "obstructed"
- [ ] Implement ["Wandering"](https://youtu.be/6BrZryMz-ac?t=323) behavior (basically an idle animation)
- [ ] Blend between fleeing and wandering depending on player proximity (with LOS)
