using SEIT = Skylark.Enum.InputType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEAEI = Sucrose.Shared.Engine.Aurora.Extension.Interaction;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEEH = Sucrose.Shared.Engine.Event.Handler;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.Aurora.Event
{
    internal static class Application
    {
        public static async void ApplicationEngine()
        {
            await Task.Delay(3000);

            SSEMI.Applications.ForEach(Application =>
            {
                IntPtr Handle = SWNM.FindWindowEx(IntPtr.Zero, IntPtr.Zero, "Engine", null);

                if (Equals(Handle, IntPtr.Zero))
                {
                    Handle = SWNM.FindWindowEx(FindWindowByProcessId(Application.Process.Id), IntPtr.Zero, "Button", "Play!");

                    if (!Equals(Handle, IntPtr.Zero))
                    {
                        SWNM.SendMessage(Handle, SWNM.BM_CLICK, IntPtr.Zero, IntPtr.Zero);
                    }
                }

                if (!Equals(Handle, IntPtr.Zero))
                {
                    SSEEH.EngineLoaded(Handle);
                    SSEEH.EngineRendered(Handle);
                }
            });

            if (SMMM.InputType != SEIT.Close)
            {
                SSEAEI.Register();
            }
        }

        public static IntPtr FindWindowByProcessId(int ProcessId)
        {
            IntPtr Result = IntPtr.Zero;

            SWNM.EnumWindows(new SWNM.EnumWindowsProc((TopHandle, TopParamHandle) =>
            {
                SWNM.GetWindowThreadProcessId(TopHandle, out int CurrentId);

                if (CurrentId == ProcessId)
                {
                    if (SWNM.IsWindowVisible(TopHandle))
                    {
                        Result = TopHandle;
                        return false;
                    }
                }

                return true;
            }), IntPtr.Zero);

            return Result;
        }
    }
}