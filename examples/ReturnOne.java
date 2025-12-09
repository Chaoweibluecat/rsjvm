/**
 * 最简单的可运行例子
 * 这个可以用当前的解释器运行！
 */
public class ReturnOne {
    // 最简单：直接返回1
    public static int returnOne() {
        return 1;
    }

    // 稍微复杂：计算后返回
    public static int addOne() {
        int a = 1;
        int b = 0;
        return a + b;
    }

    // 更复杂：多个运算
    public static int calculate() {
        int a = 10;
        int b = 20;
        int c = a + b;  // c = 30
        return c;
    }
}
