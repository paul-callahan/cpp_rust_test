#include <iostream>
#include "example.h"
#include <thread>

int THREAD_COUNT = 5;

using namespace std;
void* sr;

void task() {
    while (true) {
        send_msg(sr);
    }
}

int main() {
    sr = create_sender_receiver();
    send_msg(sr);

    thread t[THREAD_COUNT];

    for (int i = 0; i < THREAD_COUNT; i++) {
        t[i] = thread(task);
    }

    cout << THREAD_COUNT << " threads started." << endl;
    t[0].join();
    system("read");
    return 0;
}
/*
 *
 *

 bound:  1000
 1-thread throughput:     2200 msgs/ms  - no drops.
 100-thread throughput:    200 msgs/ms  - 40,000,000 drops for every 100,000 accepted.
 */
