using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
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
            if (!SSSHP.Work(SMMRA.Backgroundog) && SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }
        }
    }
}