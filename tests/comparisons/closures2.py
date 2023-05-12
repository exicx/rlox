a = "global"

def getclos():
    return lambda: print(a)

# Create a closure of the captured environment
p = getclos()

# Call p() before modifying the varible, prints "global"
p()

# We would expect p() to have captured its environment and not
# change its behavior when modifying the environment.
# But this doesn't work in python, this prints "block".
a = "block"
p()
