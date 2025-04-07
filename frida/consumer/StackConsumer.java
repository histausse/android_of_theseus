package theseus.android;

import java.util.function.Consumer;
import java.lang.StackWalker.StackFrame;
import java.util.ArrayList;
import java.util.function.Function;
import java.util.stream.Stream;

class StackConsumer implements Consumer<StackFrame> {

    public ArrayList<StackFrame> stack;

    public StackConsumer() {
        this.stack = new ArrayList<StackFrame>();
    }

    @Override
    public void accept(StackFrame frame) {
        stack.add(frame);
    }

    public StackFrame[] getStack() {
        return this.stack.toArray(new StackFrame[] {});
    }

    public Function<? super Stream<StackWalker.StackFrame>, StackFrame[]> walkNFrame(int n) {
        return s -> { s.limit(n).forEach(this); return this.getStack(); };
    }
}
