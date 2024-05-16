using SEIT = Skylark.Enum.InputType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEAEI = Sucrose.Shared.Engine.Aurora.Extension.Interaction;

namespace Sucrose.Shared.Engine.Aurora.Event
{
    internal static class Application
    {
        public static void ApplicationEngine()
        {
            if (SMMM.InputType != SEIT.Close)
            {
                SSEAEI.Register();
            }
        }
    }
}