using System.Diagnostics;
using System.Management;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Management
    {
        public static string GetCommandLine(Process Process)
        {
            try
            {
                using ManagementObjectSearcher Searcher = new("SELECT CommandLine FROM Win32_Process WHERE ProcessId = " + Process.Id);
                using ManagementObjectCollection Collection = Searcher.Get();

                return Collection.Cast<ManagementBaseObject>().SingleOrDefault()?["CommandLine"]?.ToString();
            }
            catch
            {
                return null;
            }
        }

        public static int Check(ManagementObject Object, string Title, int Back)
        {
            try
            {
                string Value = $"{Object[Title]}";

                if (string.IsNullOrEmpty(Value) || string.IsNullOrWhiteSpace(Value))
                {
                    return Back;
                }
                else
                {
                    return Convert.ToInt32(Value.TrimStart().TrimEnd());
                }
            }
            catch
            {
                return Back;
            }
        }

        public static long Check(ManagementObject Object, string Title, long Back)
        {
            try
            {
                string Value = $"{Object[Title]}";

                if (string.IsNullOrEmpty(Value) || string.IsNullOrWhiteSpace(Value))
                {
                    return Back;
                }
                else
                {
                    return Convert.ToInt64(Value.TrimStart().TrimEnd());
                }
            }
            catch
            {
                return Back;
            }
        }

        public static string Check(ManagementObject Object, string Title, string Back)
        {
            try
            {
                string Value = $"{Object[Title]}";

                if (string.IsNullOrEmpty(Value) || string.IsNullOrWhiteSpace(Value))
                {
                    return Back;
                }
                else
                {
                    return Value.TrimStart().TrimEnd();
                }
            }
            catch
            {
                return Back;
            }
        }
    }
}