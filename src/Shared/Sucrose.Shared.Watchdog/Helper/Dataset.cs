using System.Collections;
using SSWMI = Sucrose.Shared.Watchdog.Manage.Internal;

namespace Sucrose.Shared.Watchdog.Helper
{
    internal static class Dataset
    {
        public static bool Any()
        {
            return SSWMI.Dataset.Count > 0;
        }

        public static void Clear()
        {
            SSWMI.Dataset.Clear();
        }

        public static Hashtable Get()
        {
            return SSWMI.Dataset;
        }

        public static object Get(string Key)
        {
            return SSWMI.Dataset[Key];
        }

        public static void Remove(string Key)
        {
            SSWMI.Dataset.Remove(Key);
        }

        public static bool Contains(string Key)
        {
            return SSWMI.Dataset.Contains(Key);
        }

        public static void Add(string Key, object Value)
        {
            SSWMI.Dataset.Add(Key, Value);
        }

        public static void Set(string Key, object Value)
        {
            SSWMI.Dataset[Key] = Value;
        }
    }
}