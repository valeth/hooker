require "spec_helper"
require "discord_hooks"

RSpec.describe DiscordHooks do
  context "push hook" do
    let(:gl_push) { objectified_fixture("gl_push.json") }
    let(:expected) do
      desc = <<~COMMITS
        [`679ac842`](https://gitlab.com/testmaster/project/commit/679ac842ad4e77a9) More fixes - **Testmaster**
        [`45280848`](https://gitlab.com/testmaster/project/commit/4528084858866822) Fixed stuff - **Testmaster**
        [`89e617d5`](https://gitlab.com/testmaster/project/commit/89e617d5b82ee14c) Why is everything broken? - **Testmaster**
        [`df9eb970`](https://gitlab.com/testmaster/project/commit/df9eb9704fa4cf59) Added some stuff - **Testmaster**
        [`244a1db7`](https://gitlab.com/testmaster/project/commit/244a1db7f5de8052) Witty commit message - **Testmaster**
      COMMITS
      {
        author: { name: "Testmaster", icon_url: "http://example.com/testmaster.png" },
        title: "Project - 5 new commits",
        url: "https://gitlab.com/testmaster/project",
        description: desc.chomp,
        color: 0xFC6D26,
        footer: { text: "testmaster/project", icon_url: "https://gitlab.com/testmaster/project/avatar.png" }
      }
    end

    it "creates an embed on push" do
      expect(described_class.push_hook(gl_push)).to eq(expected)
    end
  end

  context "merge hook" do
    let(:gl_mr_open) { objectified_fixture("gl_mr_open.json") }
    let(:gl_mr_close) do
      gl_mr_open.dup.tap do |x|
        x.object_attributes.action = "close"
        x.object_attributes.state = "closed"
      end
    end
    let(:gl_mr_merge) do
      gl_mr_open.dup.tap do |x|
        x.object_attributes.action = "merge"
        x.object_attributes.state = "merged"
      end
    end
    let(:expected_open) do
      {
        author: { name: "Testmaster", icon_url: "http://example.com/testmaster.png" },
        title: "Project - Merge request opened: !4 Implement anti-cheat system",
        url: "https://gitlab.com/testmaster/project/merge_requests/4",
        description: "Add a anti-cheat system to keep those cheaters in check.\nAuto-ban included!",
        color: 0xE24329,
        footer: { text: "testmaster/project", icon_url: "https://gitlab.com/testmaster/project/avatar.png" },
        timestamp: Time.parse("2018-06-19 12:28:46 UTC").iso8601
      }
    end
    let(:expected_close) do
      expected_open.dup.tap do |x|
        x[:title] = "Project - Merge request closed: !4 Implement anti-cheat system"
      end
    end
    let(:expected_merge) do
      expected_open.dup.tap do |x|
        x[:title] = "Project - Merge request merged: !4 Implement anti-cheat system"
      end
    end

    it "creates an embed on opened requests" do
      expect(described_class.merge_request_hook(gl_mr_open)).to eq(expected_open)
    end

    it "creates an embed on closed requests" do
      expect(described_class.merge_request_hook(gl_mr_close)).to eq(expected_close)
    end

    it "creates an embed on merged requests" do
      expect(described_class.merge_request_hook(gl_mr_merge)).to eq(expected_merge)
    end
  end
end
