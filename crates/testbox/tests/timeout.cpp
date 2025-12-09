#include <bits/stdc++.h>
using namespace std;

int main() {
    int x = 0;
    for (int i = 1; i <= 1e9; i++) { x ^= rand(); }
    cout << x << endl;
    return 0;
}
