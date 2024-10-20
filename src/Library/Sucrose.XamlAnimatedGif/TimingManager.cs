using System.Windows.Media.Animation;

namespace Sucrose.XamlAnimatedGif
{
    class TimingManager
    {
        private readonly List<TimeSpan> _timeSpans = new();
        private int _current;
        private int _count;
        private TimeSpan _elapsed;

        public TimingManager(RepeatBehavior repeatBehavior)
        {
            RepeatBehavior = repeatBehavior;
        }

        public RepeatBehavior RepeatBehavior { get; set; }

        public void Add(TimeSpan timeSpan)
        {
            _timeSpans.Add(timeSpan);
        }

        public async Task<bool> NextAsync(CancellationToken cancellationToken)
        {
            if (IsComplete)
            {
                return false;
            }

            await IsPausedAsync(cancellationToken);

            RepeatBehavior repeatBehavior = RepeatBehavior;

            TimeSpan ts = _timeSpans[_current];
            await Task.Delay(ts, cancellationToken);
            _current++;
            _elapsed += ts;

            if (repeatBehavior.HasDuration)
            {
                if (_elapsed >= repeatBehavior.Duration)
                {
                    IsComplete = true;
                    return false;
                }
            }

            if (_current >= _timeSpans.Count)
            {
                _count++;
                if (repeatBehavior.HasCount)
                {
                    if (_count < repeatBehavior.Count)
                    {
                        _current = 0;
                        return true;
                    }
                    IsComplete = true;
                    return false;
                }
                else
                {
                    _current = 0;
                    return true;
                }
            }
            return true;
        }

        public void Reset()
        {
            _current = 0;
            _count = 0;
            _elapsed = TimeSpan.Zero;
            IsComplete = false;
        }

        public event EventHandler Completed;

        protected virtual void OnCompleted()
        {
            Completed?.Invoke(this, EventArgs.Empty);
        }

        public bool IsComplete
        {
            get;
            private set
            {
                field = value;
                if (value)
                {
                    OnCompleted();
                }
            }
        }

        private readonly Task _completedTask = Task.FromResult(0);
        private TaskCompletionSource<int> _pauseCompletionSource;
        public void Pause()
        {
            if (IsPaused)
            {
                return; // Make this a no-op.
            }

            IsPaused = true;
            _pauseCompletionSource = new TaskCompletionSource<int>();
        }

        public void Resume()
        {
            if (!IsPaused)
            {
                return; // Make this a no-op.
            }

            TaskCompletionSource<int> tcs = _pauseCompletionSource;
            tcs?.TrySetResult(0);
            _pauseCompletionSource = null;
            IsPaused = false;
        }

        public bool IsPaused { get; private set; }

        private Task IsPausedAsync(CancellationToken cancellationToken)
        {
            TaskCompletionSource<int> tcs = _pauseCompletionSource;
            if (tcs != null)
            {
                return tcs.Task.WithCancellationToken(cancellationToken);
            }
            return _completedTask;
        }
    }
}