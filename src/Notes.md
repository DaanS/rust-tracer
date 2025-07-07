# TODO

 - sort some stuff into subdirs/modules/?

## multicore

 - define a queueable job
    - something that's movable and callable
    - image region definition
    - function to get pixel color
    - place to write colors
    - what's faster: strip, tile, buffered tile?
 - make a threadsafe jobqueue
    - lifo, pop-/push_back
 - fill job queue
 - spawn worker threads
    - grabs a reference to the jobqueue and a logfile
 - spawn progress indicator thread?
 - join worker threads
 - stop progress indicator?
 - image processing