#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <inttypes.h>
#include <unistd.h>
#include <time.h>

// Declare all of these - currently, only void to void is supported
typedef struct scheduler scheduler_t;
typedef struct job job_t;
typedef void (*unit_to_unit_t)(void);

extern scheduler_t *scheduler_new(void);
extern void scheduler_free(scheduler_t *);

extern void run(job_t *, scheduler_t *, unit_to_unit_t);
extern void run_pending(scheduler_t *);

// Declare one or both of these
extern job_t *every(uint32_t);
extern job_t *every_single(void);

// Declare one of these for each method you need
extern job_t *seconds(job_t *);
extern job_t *minute(job_t *);

// Helper function to grab the current time
char *now()
{
    time_t rawtime;
    struct tm *timeinfo;
    time(&rawtime);
    timeinfo = localtime(&rawtime);
    return asctime(timeinfo);
}

// Define a job
void job(void)
{
    printf("Hello!  It is now %s\n", now());
    fflush(stdout);
}

// // NOTE: not sure how to do this - can't use generic interface arguments.
// void greet(char *name)
// {
//     printf("Hello, %s!  It's now %s", name, now());
// }

// You can't return anything, must be void return type

int main(void)
{
    printf("Starting at %s\n", now());
    
    // Instantiate
    scheduler_t *scheduler = scheduler_new();

    // Schedule some jobs - it's a little inside-out
    run(seconds(every(8)), scheduler, job);
    run(minute(every_single()), scheduler, job);

    // Run some jobs
    for (int i = 0; i < 100; i++)
    {
        run_pending(scheduler);
        sleep(1);
    }

    // Free
    scheduler_free(scheduler);
}
