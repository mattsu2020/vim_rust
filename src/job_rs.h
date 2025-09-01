#ifndef JOB_RS_H
#define JOB_RS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Job Job;

Job *job_start(const char *cmd);
int job_stop(Job *job);
int job_status(Job *job);

#ifdef __cplusplus
}
#endif

#endif // JOB_RS_H
