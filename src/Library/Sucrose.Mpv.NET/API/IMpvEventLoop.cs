namespace Sucrose.Mpv.NET.API
{
    public interface IMpvEventLoop
    {
        bool IsRunning { get; }

        Action<MpvEvent> Callback { get; set; }

        IMpvFunctions Functions { get; set; }

        void Start();
        void Stop();
    }
}