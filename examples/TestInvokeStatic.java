public class TestInvokeStatic {
    int a = 1;
    int b = 2;
    int c = 0;

    public static void main(String[] args) {
        int a = sum_a_and_b(199, 299);
    }

    public static int sum_a_and_b(int a, int b) {
        return a + b;
    }

}
