#include <bits/stdc++.h>
using namespace std;

int main() {
    system("ulimit -a");

    int x;
    cin >> x;
    printf("target to use %d mb memory\n", x);
    vector<int> unit(1 << 18);
    vector<vector<int>> mem;
    while (x--) {
        mem.push_back(unit);
        printf("memory used %d\n",
               (int)(unit.capacity() * (mem.size() + 1)) * 4 / 1024 / 1024);
        fflush(stdout);
    }
    printf("safely exit");
    return 0;
}
