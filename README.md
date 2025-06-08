ToDo:
Done //// 1. Add relays table (uuid|relayId), and db functions for it
Done //// 2. Implement last operations (related to relays)
. Proper logging
. Add RaspPI library to control GPIO and tie it up to relay operations
    add base functions open/close
   call these functions in operate
. Put relay operations under feature flag
. Add Web server, with port config, TLS (self certificate)
. map public operations (register, operate) with web server post methods
. Install and run on rasp pi zero
. logrotate
. move to electronics (buy relays, wire up)
. install against ICT server, configure ICT server (if needed, hopefully just piggy back on current config)
. implement iOS/watch app as client
. 
