using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUSI = Skylark.Wing.Utility.SingleInstance;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Instance
    {
        private static Mutex _Mutex = null;

        public static bool Basic(string Name, string Application)
        {
            try
            {
                _Mutex = new Mutex(true, Name, out bool createdNew);

                return createdNew && SSSHP.WorkCount(Application) <= 1;
            }
            catch
            {
                try
                {
                    _Mutex = new Mutex(true, Name, out bool createdNew);

                    return createdNew && SSSHP.WorkCount(Application) <= 1;
                }
                catch
                {
                    try
                    {
                        _Mutex = new Mutex(true, Name, out bool createdNew);

                        return createdNew && SSSHP.WorkCount(Application) <= 1;
                    }
                    catch
                    {
                        return false;
                    }
                }
            }
        }

        public static bool Single(string Name, string Application)
        {
#if NET6_0_OR_GREATER
            return SWUSI.IsAppMutexRunning(Name) && SSSHP.WorkCount(Application) <= 1;
#else
            _Mutex = new(true, Name);

            try
            {
                return _Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(Application) <= 1;
            }
            catch
            {
                try
                {
                    return _Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(Application) <= 1;
                }
                catch
                {
                    try
                    {
                        return _Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(Application) <= 1;
                    }
                    catch
                    {
                        return false;
                    }
                }
            }
#endif
        }
    }
}