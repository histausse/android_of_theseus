package theseus.android;

import java.util.function.Consumer;
import java.lang.StackWalker.StackFrame;
import java.util.ArrayList;

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
}
