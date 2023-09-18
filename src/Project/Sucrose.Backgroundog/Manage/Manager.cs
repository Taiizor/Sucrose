using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Manager
    {
        public static Mutex Mutex => new(true, SMR.BackgroundogMutex);
    }
}