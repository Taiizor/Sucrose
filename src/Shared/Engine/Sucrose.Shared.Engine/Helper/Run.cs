using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Run
    {
        public static bool Check()
        {
            int Result = 0;

            foreach (KeyValuePair<SSDEET, string> Pair in SSSMI.EngineLive)
            {
                if (SSSHP.Work(Pair.Value))
                {
                    Result += SSSHP.WorkCount(SSSMI.EngineLive[Pair.Key]);
                }
            }

            return Result <= 1;
        }

        public static void Control()
        {
            if (!SSSHP.Work(SMMRA.Backgroundog) && SMMB.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Backgroundog}{SMMRG.ValueSeparator}{SSSMI.Backgroundog}");
            }
        }
    }
}