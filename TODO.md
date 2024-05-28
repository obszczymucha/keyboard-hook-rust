# TODO

## Use cases

* <leader> should timeout and cancel if no follow up keys are pressed
* <leader> <unmapped key> should cancel
* <leader> <a> should immediately action EnableAllChannels
* <leader> {1-5} should trigger timeout after each follow up key (if keys
  cancel each other out - it should cancel)
* <leader> {j,k} should immediately action GainDown/GainUp and trigger timeout
  after each follow up key 
* <leader> <s> <1-7> should invoke timeout after <s> and action immediately
  when a number is pressed (this requires persistence of the current strip) 

