总答案是 

$$2^y \sum_{i=0}^k (-1)^i \binom{k}{i} 2^{x-i}(x-i)! 
$$

终于写完了...

## 代码

核心代码如下，完整代码请自行访问[剪贴板](https://www.luogu.me/paste/1yh4bevk) 或 [提交记录](https://www.luogu.com.cn/record/248616263)

```cpp
mint fc[N], fcv[N], inv[N], pw2[N];
mint binom(int n, int m) { return fc[n] * fcv[n - m] * fcv[m]; }

struct E {
    int u, v, vis;
    int get(int x) { return u ^ v ^ x; }
} e[N];

int n, p[N];
ve(int) g[N];
bool vis[N];
int tim, cnt;
int ring, ring2, chain;

int deg[N];
void dfs(int u, int fa, int &tp) {
    ++cnt, vis[u] = 1;
    for (int ei : g[u]) {
        if (e[ei].vis) { continue; }
        e[ei].vis = 1;
        int v = e[ei].get(u);
        if (v == fa) {
            tp = 1;
            deg[v]--, deg[u]--;
        } else {
            if (vis[v]) {
                tp = 2;
            } else {
                dfs(v, u, tp);
            }
        }
    }
}

void task() {
    n = rd();

    L(i, 1, n) {
        int x = rd();
        p[i] = x;
        e[i] = {i, x, 0};
        g[i].push_back(i), g[x].push_back(i);
    }
    L(i, 1, n) { deg[i] = siz(g[i]); }
    L(i, 1, n) {
        if (!vis[i]) {
            ++tim, cnt = 0;
            int tp = 0;
            dfs(i, 0, tp);
            if (tp == 1) {
                chain++, ring2 += (cnt == 2);
            } else {
                ring += (cnt > 1);
            }
        }
    }
    L(i, 1, n) {
        if (deg[i] > 2) { return wr("0\n"), void(); }
    }

    mint ans(0);
    L(i, 0, ring2) {
        ans += mint(neg1(i & 1)) * binom(ring2, i) * fc[chain - i] *
               pw2[chain - i];
    }
    ans *= pw2[ring];
    wr("%lld\n", ans.v);
}
```