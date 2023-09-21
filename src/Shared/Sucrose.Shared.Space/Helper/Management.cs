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
    }
}