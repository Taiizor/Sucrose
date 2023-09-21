using System.Diagnostics;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Live
    {
        public static bool Run()
        {
            foreach (KeyValuePair<SSDEET, string> Pair in SSSMI.EngineLive)
            {
                if (SSSHP.Work(Pair.Value))
                {
                    return true;
                }
            }

            return false;
        }

        public static bool Run(SSDEET Live)
        {
            if (SSSHP.Work(SSSMI.EngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static bool Run(string Live)
        {
            if (SSSHP.Work(SSSMI.TextEngineLive[Live]))
            {
                return true;
            }

            return false;
        }

        public static Process Get()
        {
            foreach (KeyValuePair<SSDEET, string> Pair in SSSMI.EngineLive)
            {
                if (SSSHP.Work(Pair.Value))
                {
                    return SSSHP.Get(Pair.Value);
                }
            }

            return null;
        }

        public static Process Get(SSDEET Live)
        {
            if (SSSHP.Work(SSSMI.EngineLive[Live]))
            {
                return SSSHP.Get(SSSMI.EngineLive[Live]);
            }

            return null;
        }

        public static void Kill()
        {
            foreach (KeyValuePair<SSDEET, string> Pair in SSSMI.EngineLive)
            {
                if (SSSHP.Work(Pair.Value))
                {
                    SSSHP.Kill(Pair.Value);
                }
            }
        }

        public static void Kill(SSDEET Live)
        {
            SSSHP.Kill(SSSMI.EngineLive[Live]);
        }

        public static void Kill(string Live)
        {
            SSSHP.Kill(SSSMI.TextEngineLive[Live]);
        }
    }
}