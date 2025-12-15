/**
 * 最简单的main方法
 * 只做算术运算，不调用其他方法
 */
public class SimpleMain {
    int a = 1;
    int b = 2;
    int c = 0;

    public static void main(String[] args) {
        SimpleMain m = new SimpleMain();
        m.c = m.sum_a_and_b();
    }

    public int sum_a_and_b() {
        return a + b;
    }
}
