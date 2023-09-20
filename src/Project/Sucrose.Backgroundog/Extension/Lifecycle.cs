using System.Diagnostics;
using System.Runtime.InteropServices;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Lifecycle
    {
        public static bool Resume(IntPtr Handle)
        {
            int Result = NtResumeProcess(Handle);

            if (Result == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool Resume(Process Process)
        {
            return Resume(Process.Handle);
        }

        public static bool Suspend(IntPtr Handle)
        {
            int Result = NtSuspendProcess(Handle);

            if (Result == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool Suspend(Process Process)
        {
            return Suspend(Process.Handle);
        }

        public static void ResumeThread(Process Process)
        {
            foreach (ProcessThread Thread in Process.Threads)
            {
                IntPtr Threading = OpenThread(SBMI.THREAD_SUSPEND_RESUME, false, Thread.Id);

                ResumeThread(Threading);
            }
        }

        public static void SuspendThread(Process Process)
        {
            foreach (ProcessThread Thread in Process.Threads)
            {
                IntPtr Threading = OpenThread(SBMI.THREAD_SUSPEND_RESUME, false, Thread.Id);

                SuspendThread(Threading);
            }
        }

        [DllImport("kernel32.dll")]
        private static extern int ResumeThread(IntPtr hThread);

        [DllImport("kernel32.dll")]
        private static extern uint SuspendThread(IntPtr hThread);

        [DllImport("ntdll.dll")]
        private static extern int NtResumeProcess(IntPtr ProcessHandle);

        [DllImport("ntdll.dll")]
        private static extern int NtSuspendProcess(IntPtr ProcessHandle);

        [DllImport("kernel32.dll")]
        private static extern IntPtr OpenThread(int dwDesiredAccess, bool bInheritHandle, int dwThreadId);
    }
}