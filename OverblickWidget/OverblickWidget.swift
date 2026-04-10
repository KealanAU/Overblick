import WidgetKit
import SwiftUI

// ---------------------------------------------------------------------------
// Data model — mirrors Rust's WidgetData struct
// ---------------------------------------------------------------------------

struct WidgetData: Codable {
    var today_minutes: Int
    var week_minutes: Int
    var recent_commits: [WidgetCommit]
    var updated_at: TimeInterval
}

struct WidgetCommit: Codable {
    var repo: String
    var message: String
    var author: String
    var committed_at: TimeInterval
}

// ---------------------------------------------------------------------------
// Data loading
// ---------------------------------------------------------------------------

func dataFilePath() -> URL {
    let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
    return appSupport.appendingPathComponent("overblick/widget_data.json")
}

func loadWidgetData() -> WidgetData {
    let fallback = WidgetData(today_minutes: 0, week_minutes: 0, recent_commits: [], updated_at: 0)
    guard let data = try? Data(contentsOf: dataFilePath()),
          let decoded = try? JSONDecoder().decode(WidgetData.self, from: data)
    else { return fallback }
    return decoded
}

func formatMinutes(_ minutes: Int) -> String {
    if minutes == 0 { return "0m" }
    let h = minutes / 60
    let m = minutes % 60
    if h == 0 { return "\(m)m" }
    if m == 0 { return "\(h)h" }
    return "\(h)h \(m)m"
}

// ---------------------------------------------------------------------------
// Widget timeline
// ---------------------------------------------------------------------------

struct OverblickEntry: TimelineEntry {
    let date: Date
    let data: WidgetData
}

struct OverblickProvider: TimelineProvider {
    func placeholder(in context: Context) -> OverblickEntry {
        OverblickEntry(date: Date(), data: WidgetData(today_minutes: 90, week_minutes: 420, recent_commits: [], updated_at: 0))
    }

    func getSnapshot(in context: Context, completion: @escaping (OverblickEntry) -> Void) {
        completion(OverblickEntry(date: Date(), data: loadWidgetData()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<OverblickEntry>) -> Void) {
        let entry = OverblickEntry(date: Date(), data: loadWidgetData())
        // Refresh every 15 minutes
        let next = Calendar.current.date(byAdding: .minute, value: 15, to: Date())!
        completion(Timeline(entries: [entry], policy: .after(next)))
    }
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

struct OverblickWidgetSmallView: View {
    let data: WidgetData

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 4) {
                Image(systemName: "clock.fill")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                Text("Överblick")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Text(formatMinutes(data.today_minutes))
                .font(.title2.bold())

            Text("today")
                .font(.caption)
                .foregroundStyle(.secondary)

            Text(formatMinutes(data.week_minutes) + " this week")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
    }
}

struct OverblickWidgetMediumView: View {
    let data: WidgetData

    var body: some View {
        HStack(spacing: 0) {
            // Time summary
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 4) {
                    Image(systemName: "clock.fill")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text("Överblick")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text(formatMinutes(data.today_minutes))
                    .font(.title.bold())
                Text("today")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(formatMinutes(data.week_minutes) + " this week")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            .padding()
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)

            Divider().padding(.vertical, 12)

            // Recent commits
            VStack(alignment: .leading, spacing: 4) {
                Text("Recent commits")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.bottom, 2)

                if data.recent_commits.isEmpty {
                    Text("No commits yet")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                } else {
                    ForEach(Array(data.recent_commits.prefix(3).enumerated()), id: \.offset) { _, commit in
                        VStack(alignment: .leading, spacing: 1) {
                            Text(commit.message)
                                .font(.caption2.weight(.medium))
                                .lineLimit(1)
                            Text(commit.repo)
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                                .lineLimit(1)
                        }
                    }
                }
                Spacer()
            }
            .padding()
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        }
    }
}

struct OverblickWidgetEntryView: View {
    @Environment(\.widgetFamily) var family
    let entry: OverblickEntry

    var body: some View {
        switch family {
        case .systemSmall:
            OverblickWidgetSmallView(data: entry.data)
        case .systemMedium:
            OverblickWidgetMediumView(data: entry.data)
        default:
            OverblickWidgetSmallView(data: entry.data)
        }
    }
}

// ---------------------------------------------------------------------------
// Widget definition
// ---------------------------------------------------------------------------

@main
struct OverblickWidget: Widget {
    let kind = "OverblickWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: OverblickProvider()) { entry in
            OverblickWidgetEntryView(entry: entry)
                .containerBackground(.fill.tertiary, for: .widget)
        }
        .configurationDisplayName("Överblick")
        .description("Time logged and recent commits.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}
