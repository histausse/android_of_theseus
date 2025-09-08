package com.example.theseus;

import android.app.Activity;
import android.util.Base64;
import android.util.Log;
import dalvik.system.InMemoryDexClassLoader;
import java.lang.Class;
import java.lang.ClassLoader;
import java.nio.ByteBuffer;

import javax.crypto.Cipher;
import javax.crypto.spec.SecretKeySpec;
import java.security.Key;

public class Main {

    private static final String DEX = "ZGV4CjAzNQAvtfp88wvPHZi+B2FO1HZ02SatJfZGyA94BAAAcAAAAHhWNBIAAAAAAAAAANgDAAAVAAAAcAAAAAcAAADEAAAABQAAAOAAAAAAAAAAAAAAAAgAAAAcAQAAAQAAAFwBAAD8AgAAfAEAACACAAAoAgAAKwIAAC8CAAA0AgAATAIAAG0CAACKAgAAngIAALICAADNAgAA3QIAAOoCAADtAgAA8gIAAPUCAAD9AgAABwMAABIDAAAYAwAAIgMAAAQAAAAFAAAABgAAAAcAAAAIAAAACQAAAAwAAAABAAAABAAAAAAAAAADAAAABAAAAAgCAAACAAAABQAAABACAAAMAAAABgAAAAAAAAANAAAABgAAABgCAAABAAMAAAAAAAEAAQAQAAAAAQABABEAAAACAAQAEgAAAAMAAwAAAAAABQADAAAAAAAFAAIADwAAAAUAAAATAAAAAQAAAAEAAAADAAAAAAAAAAoAAAAAAAAAwAMAAAAAAAADAAIAAgAAAPQBAAAaAAAAIgIFAHAQBQACABoACwBuIAYAAgAMAm4gBgASAAwBGgIOAG4gBgAhAAwBbhAHAAEADAERAQIAAgACAAAA+gEAAAUAAABxIAMAAQASABEAAAABAAEAAQAAAAECAAAEAAAAcBAEAAAADgAGAgAADgAKAgAADjwABAAOAAAAAAIAAAAEAAAAAQAAAAQAAAACAAAAAAAEAAY8aW5pdD4AAUwAAkxMAANMTEwAFkxhbmRyb2lkL2FwcC9BY3Rpdml0eTsAH0xjb20vZXhhbXBsZS90aGVzZXVzL01hbGljaW91czsAG0xjb20vZXhhbXBsZS90aGVzZXVzL1V0aWxzOwASTGphdmEvbGFuZy9PYmplY3Q7ABJMamF2YS9sYW5nL1N0cmluZzsAGUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsADk1hbGljaW91cy5qYXZhAAtTZWNyZXREYXRhWwABVgADVkxMAAFdAAZhcHBlbmQACGdldF9kYXRhAAlzZW5kX2RhdGEABHNpbmsACHRvU3RyaW5nAJsBfn5EOHsiYmFja2VuZCI6ImRleCIsImNvbXBpbGF0aW9uLW1vZGUiOiJkZWJ1ZyIsImhhcy1jaGVja3N1bXMiOmZhbHNlLCJtaW4tYXBpIjoxLCJzaGEtMSI6ImZhY2VkZjQxYmJkMjhiNTYzZDFlOWUwOWM1ZjcyZDdjNWNhNTk4ZDUiLCJ2ZXJzaW9uIjoiOC4yLjItZGV2In0AAAADAACBgATcAwEJ/AIBCcADAAAAAAAADQAAAAAAAAABAAAAAAAAAAEAAAAVAAAAcAAAAAIAAAAHAAAAxAAAAAMAAAAFAAAA4AAAAAUAAAAIAAAAHAEAAAYAAAABAAAAXAEAAAEgAAADAAAAfAEAAAMgAAADAAAA9AEAAAEQAAADAAAACAIAAAIgAAAVAAAAIAIAAAAgAAABAAAAwAMAAAMQAAABAAAA1AMAAAAQAAABAAAA2AMAAA==";
    private Key key;
    ClassLoader cl;
    Activity ac;


    public Main(Activity ac) throws Exception {
        this.key = new SecretKeySpec("_-_Secret Key_-_".getBytes(), "AES");
        this.ac = ac;
        byte[] bytes = Base64.decode(DEX, Base64.NO_WRAP);
        this.cl = new InMemoryDexClassLoader(ByteBuffer.wrap(bytes), Main.class.getClassLoader());
    }

    public void main() throws  Exception {
        String[] methods = {"n6WGYJzjDrUvR9cYljlNlw==", "dapES0wl/iFIPuMnH3fh7g=="};
        Class cls = cl.loadClass(decrypt("W5f3xRf3wCSYcYG7ckYGR5xuuESDZ2NcDUzGxsq3sls="));

        Object val = "imei";

        for (String method : methods) {
            val = cls.getMethod(decrypt(method), String.class, Activity.class).invoke(null, val, ac);
        }
    }

    public String encrypt(String s) throws  Exception {
        Cipher c = Cipher.getInstance("AES/ECB/PKCS5Padding");// Doubious choise of encryption but it's just a demo
        c.init(Cipher.ENCRYPT_MODE, key);
        byte[] cpt = c.doFinal(s.getBytes());
        return Base64.encodeToString(cpt, Base64.NO_WRAP);
    }

    public String decrypt(String s) throws  Exception {
        Cipher c = Cipher.getInstance("AES/ECB/PKCS5Padding");// Doubious choise of encryption but it's just a demo
        c.init(Cipher.DECRYPT_MODE, key);
        byte[] clt = c.doFinal(Base64.decode(s, Base64.NO_WRAP));
        return new String(clt);
    }
}
