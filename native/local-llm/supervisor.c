// Own an Ollama process group and reap it when H exits, including abnormal exit.
#include <signal.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>
#include <errno.h>
static volatile sig_atomic_t stopped = 0;
static void stop(int sig) { (void)sig; stopped = 1; }
int main(int argc, char **argv) {
    if (argc != 3) return 64;
    const pid_t owner = (pid_t)strtol(argv[1], NULL, 10);
    if (owner <= 1 || getppid() != owner) return 65;
    signal(SIGTERM, stop); signal(SIGINT, stop);
    pid_t child = fork();
    if (child < 0) return 66;
    if (child == 0) {
        setpgid(0, 0);
        execl(argv[2], argv[2], "serve", (char *)NULL);
        _exit(127);
    }
    setpgid(child, child);
    int status = 0;
    while (!stopped && getppid() == owner) {
        pid_t result = waitpid(child, &status, WNOHANG);
        if (result == child || (result < 0 && errno != EINTR)) break;
        usleep(100000);
    }
    kill(-child, SIGTERM);
    for (int i=0; i<10; i++) usleep(100000);
    kill(-child, SIGKILL);
    while (waitpid(child, &status, 0) < 0 && errno == EINTR) {}
    return 0;
}
