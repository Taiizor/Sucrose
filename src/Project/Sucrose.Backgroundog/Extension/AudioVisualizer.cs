using MathNet.Numerics.IntegralTransforms;
using NAudio.CoreAudioApi;
using NAudio.CoreAudioApi.Interfaces;
using NAudio.Wave;
using System.Diagnostics;
using System.Numerics;

namespace Sucrose.Backgroundog.Extension
{
    internal class AudioVisualizer : IMMNotificationClient
    {
        public event EventHandler<double[]> AudioDataAvailable;

        private readonly int MaxSample = 128;
        private WasapiLoopbackCapture Capture;
        private readonly int VerticalSmoothness = 2;
        private readonly int HorizontalSmoothness = 1;
        private readonly List<Complex[]> Smooth = new();
        private readonly MMDeviceEnumerator DeviceEnum = new();

        public AudioVisualizer()
        {
            try
            {
                int HRESULT = DeviceEnum.RegisterEndpointNotificationCallback(this);

                if (HRESULT != 0)
                {
                    Debug.WriteLine("Failed to register audio device notifications.");
                }

                Capture = CreateWasapiLoopbackCapture();
            }
            catch (Exception Exception)
            {
                Debug.WriteLine($"Failed to initialize audio visualizer: {Exception.Message}");
            }
        }

        public void Start()
        {
            Capture?.StartRecording();
        }

        public void Stop()
        {
            Capture?.StopRecording();
        }

        private WasapiLoopbackCapture CreateWasapiLoopbackCapture(MMDevice Device = null)
        {
            WasapiLoopbackCapture TempCapture = Device != null ? new WasapiLoopbackCapture(Device) : new WasapiLoopbackCapture();

            TempCapture.DataAvailable += ProcessAudioData;

            TempCapture.RecordingStopped += (s, a) =>
            {
                TempCapture?.Dispose();
            };

            return TempCapture;
        }

        private void ProcessAudioData(object sender, WaveInEventArgs e)
        {
            try
            {
                WaveBuffer Buffer = new(e.Buffer);

                int Length = Buffer.FloatBuffer.Length / 8;

                // FFT
                Complex[] Values = new Complex[Length];

                for (int C = 0; C < Length; C++)
                {
                    Values[C] = new Complex(Buffer.FloatBuffer[C], 0.0);
                }

                Fourier.Forward(Values, FourierOptions.Default);

                // Shift Array
                Smooth.Add(Values);

                if (Smooth.Count > VerticalSmoothness)
                {
                    Smooth.RemoveAt(0);
                }

                double[] AudioData = new double[MaxSample];

                for (int i = 0; i < MaxSample; i++)
                {
                    AudioData[i] = BothSmooth(i);
                }

                AudioDataAvailable?.Invoke(this, AudioData);
            }
            catch (Exception Exception)
            {
                Debug.WriteLine($"Failed to process audio data: {Exception.Message}");
            }
        }

        private double BothSmooth(int C)
        {
            Complex[][] S = Smooth.ToArray();

            double Value = 0;

            for (int H = Math.Max(C - HorizontalSmoothness, 0); H < Math.Min(C + HorizontalSmoothness, MaxSample); H++)
            {
                Value += VSmooth(H, S);
            }

            return Value / ((HorizontalSmoothness + 1) * 2);
        }

        private double VSmooth(int C, Complex[][] S)
        {
            double Value = 0;

            for (int V = 0; V < S.Length; V++)
            {
                Value += Math.Abs(S[V] != null ? S[V][C].Magnitude : 0.0);
            }

            return Value / S.Length;
        }

        public void OnDefaultDeviceChanged(DataFlow Flow, Role Role, string DefaultDeviceId)
        {
            if (Flow == DataFlow.Render)
            {
                try
                {
                    Capture?.StopRecording();

                    //MMDeviceEnumerator Enumerator = new();
                    //MMDevice DefaultDevice = Enumerator.GetDefaultAudioEndpoint(DataFlow.Render, Role.Multimedia);

                    Capture = CreateWasapiLoopbackCapture();
                    Capture.StartRecording();
                }
                catch (Exception Exception)
                {
                    Debug.WriteLine($"Failed to update WasapiLoopbackCapture device: {Exception.Message}");
                }
            }
        }

        public void OnDeviceStateChanged(string DeviceId, DeviceState NewState)
        {
            Debug.WriteLine($"Device state changed: Device Id -> {DeviceId} State -> {NewState}");
        }

        public void OnDeviceAdded(string PwstrDeviceId)
        {
            Debug.WriteLine($"Device added: {PwstrDeviceId}");
        }

        public void OnDeviceRemoved(string DeviceId)
        {
            Debug.WriteLine($"Device removed: {DeviceId}");
        }

        public void OnPropertyValueChanged(string PwstrDeviceId, PropertyKey Key)
        {
            Debug.WriteLine($"Property Value Changed: formatId -> {Key.formatId}  propertyId -> {Key.propertyId}");
        }

        public void Dispose()
        {
            DeviceEnum?.UnregisterEndpointNotificationCallback(this);
            Stop();
            //Calling dispose outside hangs.
            //Capture?.Dispose();
        }
    }
}