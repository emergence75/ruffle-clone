# SWF Regression Tests

Inside [tests/swfs](tests/swfs) is a large collection of automated tests that are based around running a swf and seeing what happens.

To create a test, make a directory that looks like the following, at minimum:

- directory/
  - test.swf
  - test.toml
  - output.txt

As best practice, please also include any source used to make the swf - such as `test.fla` and any actionscript files.


# Test Structure
## test.toml
Except for `num_frames`, every other field and section is optional.

```toml
num_frames = 1 # The amount of frames of the swf to run
sleep_to_meet_frame_rate = false # If true, slow the tick rate to match the movies requested fps rate
image = false # If true, capture a screenshot of the movie and compare it against a "known good" image
ignore = false # If true, ignore this test. Please comment why, ideally link to an issue, so we know what's up

# Sometimes floating point math doesn't exactly 100% match between flash and rust.
# If you encounter this in a test, the following section will change the output testing from "exact" to "approximate"
# (when it comes to floating point numbers, at least.)
[approximations]
number_patterns = [] # A list of regex patterns with capture groups to additionally treat as approximate numbers
epsilon = 0.0 # The upper bound of any rounding errors. Default is the difference between 1.0 and the next largest representable number
max_relative = 0.0 # The default relative tolerance for testing values that are far-apart. Default is the difference between 1.0 and the next largest representable number

# Options for the player used to run this swf 
[player_options]
max_execution_duration = { secs = 15, nanos = 0} # How long can actionscript execute for before being forcefully stopped
viewport_dimensions = { width = 100, height = 100, scale_factor = 1 } # The size of the player. Defaults to the swfs stage size
```
